#![doc = include_str!("../README.md")]

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::{
        header::{HeaderValue, ACCEPT, CONTENT_TYPE},
        StatusCode,
    },
    response::{IntoResponse, Response},
    Extension,
};
use tower::Service;

#[cfg(all(feature = "json", feature = "simd-json"))]
compile_error!("json and simd-json features are mutually exclusive");
#[cfg(all(feature = "default-json", feature = "default-cbor"))]
compile_error!("default-json and default-cbor features are mutually exclusive");

#[cfg(feature = "default-json")]
/// Default to application/json if the request does not have any accept header or uses */* when json is enabled
static DEFAULT_CONTENT_TYPE_VALUE: &str = "application/json";

#[cfg(feature = "default-cbor")]
/// Default to application/cbor if the request does not have any accept header or uses */* when json is not enabled
static DEFAULT_CONTENT_TYPE_VALUE: &str = "application/cbor";

#[cfg(not(any(feature = "default-json", feature = "default-cbor")))]
compile_error!("A default-* feature must be enabled for fallback encoding");

static DEFAULT_CONTENT_TYPE: HeaderValue = HeaderValue::from_static(DEFAULT_CONTENT_TYPE_VALUE);

static MALFORMED_RESPONSE: (StatusCode, &str) = (StatusCode::BAD_REQUEST, "Malformed request body");

#[derive(Debug, Clone)]
pub struct Negotiate<T>(T);

#[async_trait]
impl<T, S> FromRequest<S> for Negotiate<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let accept = req
            .headers()
            .get(CONTENT_TYPE)
            .unwrap_or(&DEFAULT_CONTENT_TYPE);

        match accept.as_bytes() {
            #[cfg(feature = "simd-json")]
            b"application/json" => {
                let mut body = Bytes::from_request(req, state)
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "failed to ready request body as bytes");
                        e.into_response()
                    })?
                    .to_vec();

                let body = simd_json::from_slice(&mut body).map_err(|e| {
                    tracing::error!(error = %e, "failed to deserialize request body as json");
                    MALFORMED_RESPONSE.into_response()
                })?;

                Ok(Self(body))
            }
            #[cfg(feature = "json")]
            b"application/json" => {
                let body = Bytes::from_request(req, state).await.map_err(|e| {
                    tracing::error!(error = %e, "failed to ready request body as bytes");
                    e.into_response()
                })?;

                let body = serde_json::from_slice(&body).map_err(|e| {
                    tracing::error!(error = %e, "failed to deserialize request body as json");
                    MALFORMED_RESPONSE.into_response()
                })?;

                Ok(Self(body))
            }

            #[cfg(feature = "cbor")]
            b"application/cbor" => {
                let body = Bytes::from_request(req, state).await.map_err(|e| {
                    tracing::error!(error = %e, "failed to ready request body as bytes");
                    e.into_response()
                })?;

                let body = cbor4ii::serde::from_slice(&body).map_err(|e| {
                    tracing::error!(error = %e, "failed to deserialize request body as json");
                    MALFORMED_RESPONSE.into_response()
                })?;

                Ok(Self(body))
            }

            _ => {
                tracing::error!("unsupported accept header: {:?}", accept);
                return Err((
                    StatusCode::NOT_ACCEPTABLE,
                    "Invalid content type on request",
                )
                    .into_response());
            }
        }
    }
}

#[derive(Clone)]
struct ErasedNegotiate(Arc<Box<dyn erased_serde::Serialize + Send + Sync>>);

impl<T> From<T> for ErasedNegotiate
where
    T: serde::Serialize + Send + Sync + 'static,
{
    fn from(value: T) -> Self {
        Self(Arc::new(Box::from(value)))
    }
}

impl<T> IntoResponse for Negotiate<T>
where
    T: serde::Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let data: ErasedNegotiate = self.0.into();
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Extension(data),
            "Misconfigured service layer",
        )
            .into_response()
    }
}

#[derive(Clone)]
struct NegotiateLayer;

impl<S> tower::Layer<S> for NegotiateLayer {
    type Service = NegotiateService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NegotiateService(inner)
    }
}

trait AcceptExt {
    fn negotiate(&self) -> Option<&'static str>;
}

impl AcceptExt for axum::http::HeaderMap {
    /// Basic implementation without q= values
    fn negotiate(&self) -> Option<&'static str> {
        let accept = self.get(ACCEPT).unwrap_or(&DEFAULT_CONTENT_TYPE);

        match accept.as_bytes() {
            #[cfg(any(feature = "simd-json", feature = "json"))]
            b"application/json" => Some("application/json"),
            #[cfg(feature = "cbor")]
            b"application/cbor" => Some("application/cbor"),
            b"*/*" => Some(DEFAULT_CONTENT_TYPE_VALUE),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct NegotiateService<S>(S);

impl<T> Service<Request> for NegotiateService<T>
where
    T: Service<Request>,
    T::Response: IntoResponse,
    T::Future: Send + 'static,
{
    type Response = axum::response::Response;
    type Error = T::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let accept = request.headers().negotiate();

        let Some(encoding) = accept else {
            return Box::pin(async move {
                let response: Response = (
                    StatusCode::NOT_ACCEPTABLE,
                    "Invalid content type on request",
                )
                    .into_response();
                Ok(response)
            });
        };

        let future = self.0.call(request);

        Box::pin(async move {
            let x = future.await?;
            let response: Response = x.into_response();
            let data = response.extensions().get::<ErasedNegotiate>();

            let Some(ErasedNegotiate(payload)) = data else {
                return Ok(response);
            };

            let body = match encoding {
                #[cfg(any(feature = "simd-json", feature = "json"))]
                "application/json" => {
                    let mut body = Vec::new();
                    {
                        let mut serializer = serde_json::Serializer::new(&mut body);
                        let mut serializer = <dyn erased_serde::Serializer>::erase(&mut serializer);
                        if let Err(e) = payload.erased_serialize(&mut serializer) {
                            tracing::error!(error = %e, "failed to deserialize request body as json");

                            let response: Response = (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Failed to serialize response",
                            )
                                .into_response();
                            return Ok(response);
                        };
                    }
                    body
                }
                #[cfg(feature = "cbor")]
                "application/cbor" => {
                    let mut body = cbor4ii::core::utils::BufWriter::new(Vec::new());
                    {
                        let mut serializer = cbor4ii::serde::Serializer::new(&mut body);
                        let mut serializer = <dyn erased_serde::Serializer>::erase(&mut serializer);
                        if let Err(e) = payload.erased_serialize(&mut serializer) {
                            tracing::error!(error = %e, "failed to deserialize request body as cbor");

                            let response: Response = (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Failed to serialize response",
                            )
                                .into_response();
                            return Ok(response);
                        }
                    }
                    body.into_inner()
                }
                _ => vec![],
            };

            let (mut parts, _) = response.into_parts();
            if parts.status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
                parts.status = StatusCode::OK;
            }
            parts
                .headers
                .insert(CONTENT_TYPE, HeaderValue::from_static(encoding));

            Ok(Response::from_parts(parts, body.into()))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::Negotiate;

    use axum::{
        body::Body,
        http::{
            header::{ACCEPT, CONTENT_TYPE},
            Request, StatusCode,
        },
        response::IntoResponse,
        routing::post,
        Router,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::NegotiateLayer;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Example {
        message: String,
    }

    mod general {
        use super::*;

        mod input {
            use super::*;

            #[tokio::test]
            async fn test_does_not_process_handler_if_content_type_is_not_supported() {
                #[axum::debug_handler]
                async fn handler(_: Negotiate<Example>) -> impl IntoResponse {
                    unimplemented!("This should not be called")
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .header(CONTENT_TYPE, "non-supported")
                            .method("POST")
                            .body(Body::from("really-cool-format"))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 406);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Invalid content type on request"
                );
            }
        }

        mod output {
            use super::*;

            #[tokio::test]
            async fn test_inform_error_when_misconfigured() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    Negotiate(Example {
                        message: "Hello, test!".to_string(),
                    })
                }

                let app = Router::new().route("/", post(handler));

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 415);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Misconfigured service layer"
                );
            }

            #[tokio::test]
            async fn test_does_not_process_handler_if_accept_is_not_supported() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    unimplemented!("This should not be called")
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .header(ACCEPT, "non-supported")
                            .method("POST")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 406);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Invalid content type on request"
                );
            }
        }
    }

    #[cfg(any(feature = "simd-json", feature = "json"))]
    mod json {
        use serde_json::json;

        use super::*;

        mod input {
            use super::*;

            #[cfg(feature = "default-json")]
            #[tokio::test]
            async fn test_can_read_input_without_content_type_by_default() {
                #[axum::debug_handler]
                async fn handler(Negotiate(input): Negotiate<Example>) -> impl IntoResponse {
                    format!("Hello, {}!", input.message)
                }

                let app = Router::new().route("/", post(handler));

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .body(json!({ "message": "test" }).to_string())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Hello, test!"
                );
            }

            #[tokio::test]
            async fn test_can_read_input_with_specified_header() {
                #[axum::debug_handler]
                async fn handler(Negotiate(input): Negotiate<Example>) -> impl IntoResponse {
                    format!("Hello, {}!", input.message)
                }

                let app = Router::new().route("/", post(handler));

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .header(CONTENT_TYPE, "application/json")
                            .method("POST")
                            .body(json!({ "message": "test" }).to_string())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Hello, test!"
                );
            }

            #[tokio::test]
            async fn test_does_not_accept_invalid_inputs() {
                #[axum::debug_handler]
                async fn handler(_: Negotiate<Example>) -> impl IntoResponse {
                    unimplemented!("This should not be called")
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .header(CONTENT_TYPE, "application/json")
                            .body(json!({ "not": true }).to_string())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 400);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Malformed request body"
                );
            }
        }

        mod output {
            use super::*;

            #[tokio::test]
            async fn test_encode_as_requested() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    Negotiate(Example {
                        message: "Hello, test!".to_string(),
                    })
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .header(ACCEPT, "application/json")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/json"
                );
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    json!({ "message": "Hello, test!" }).to_string()
                );
            }

            #[cfg(feature = "default-json")]
            #[tokio::test]
            async fn test_use_default_encoding_without_headers() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    Negotiate(Example {
                        message: "Hello, test!".to_string(),
                    })
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/json"
                );
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    json!({ "message": "Hello, test!" }).to_string()
                );
            }

            #[tokio::test]
            async fn test_retain_handler_status_code() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    (
                        StatusCode::CREATED,
                        Negotiate(Example {
                            message: "Hello, test!".to_string(),
                        }),
                    )
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::CREATED);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/json"
                );
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    json!({ "message": "Hello, test!" }).to_string()
                );
            }
        }
    }

    #[cfg(feature = "cbor")]
    mod cbor {
        use cbor4ii::core::{enc::Encode, utils::BufWriter, Value};

        use super::*;

        mod input {
            use super::*;

            #[cfg(feature = "default-cbor")]
            #[tokio::test]
            async fn test_can_read_input_without_content_type_by_default() {
                #[axum::debug_handler]
                async fn handler(Negotiate(input): Negotiate<Example>) -> impl IntoResponse {
                    format!("Hello, {}!", input.message)
                }

                let app = Router::new().route("/", post(handler));
                let body = {
                    let mut writer = BufWriter::new(Vec::new());
                    Value::Map(vec![(
                        Value::Text("message".to_string()),
                        Value::Text("test".to_string()),
                    )])
                    .encode(&mut writer)
                    .unwrap();
                    writer.into_inner()
                };

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Hello, test!"
                );
            }

            #[tokio::test]
            async fn test_can_read_input_with_specified_header() {
                #[axum::debug_handler]
                async fn handler(Negotiate(input): Negotiate<Example>) -> impl IntoResponse {
                    format!("Hello, {}!", input.message)
                }

                let app = Router::new().route("/", post(handler));
                let body = {
                    let mut writer = BufWriter::new(Vec::new());
                    Value::Map(vec![(
                        Value::Text("message".to_string()),
                        Value::Text("test".to_string()),
                    )])
                    .encode(&mut writer)
                    .unwrap();
                    writer.into_inner()
                };

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .header(CONTENT_TYPE, "application/cbor")
                            .method("POST")
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    "Hello, test!"
                );
            }
        }

        mod output {
            use super::*;

            #[tokio::test]
            async fn test_encode_as_requested() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    Negotiate(Example {
                        message: "Hello, test!".to_string(),
                    })
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .header(ACCEPT, "application/cbor")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/cbor"
                );
                assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), {
                    let mut writer = BufWriter::new(Vec::new());
                    Value::Map(vec![(
                        Value::Text("message".to_string()),
                        Value::Text("Hello, test!".to_string()),
                    )])
                    .encode(&mut writer)
                    .unwrap();
                    writer.into_inner()
                });
            }

            #[tokio::test]
            async fn test_retain_status_code() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    (
                        StatusCode::CREATED,
                        Negotiate(Example {
                            message: "Hello, test!".to_string(),
                        }),
                    )
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .header(ACCEPT, "application/cbor")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::CREATED);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/cbor"
                );
                assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), {
                    let mut writer = BufWriter::new(Vec::new());
                    Value::Map(vec![(
                        Value::Text("message".to_string()),
                        Value::Text("Hello, test!".to_string()),
                    )])
                    .encode(&mut writer)
                    .unwrap();
                    writer.into_inner()
                });
            }

            #[cfg(feature = "default-cbor")]
            #[tokio::test]
            async fn test_default_encoding_without_header() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    (
                        StatusCode::CREATED,
                        Negotiate(Example {
                            message: "Hello, test!".to_string(),
                        }),
                    )
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::CREATED);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/cbor"
                );
                assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), {
                    let mut writer = BufWriter::new(Vec::new());
                    Value::Map(vec![(
                        Value::Text("message".to_string()),
                        Value::Text("Hello, test!".to_string()),
                    )])
                    .encode(&mut writer)
                    .unwrap();
                    writer.into_inner()
                });
            }

            #[cfg(feature = "default-cbor")]
            #[tokio::test]
            async fn test_default_encoding_with_star() {
                #[axum::debug_handler]
                async fn handler() -> impl IntoResponse {
                    (
                        StatusCode::CREATED,
                        Negotiate(Example {
                            message: "Hello, test!".to_string(),
                        }),
                    )
                }

                let app = Router::new()
                    .route("/", post(handler))
                    .layer(NegotiateLayer);

                let response = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .method("POST")
                            .header(ACCEPT, "*/*")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::CREATED);
                assert_eq!(
                    response.headers().get(CONTENT_TYPE).unwrap(),
                    "application/cbor"
                );
                assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), {
                    let mut writer = BufWriter::new(Vec::new());
                    Value::Map(vec![(
                        Value::Text("message".to_string()),
                        Value::Text("Hello, test!".to_string()),
                    )])
                    .encode(&mut writer)
                    .unwrap();
                    writer.into_inner()
                });
            }
        }
    }
}
