# axum-content-negotiation

HTTP Content Negotiation middleware and extractor for Axum.

<a href="https://github.com/bltavares/axum-content-negotiation/actions?query=workflow%3AQuickstart+branch%3Amaster">
    <img src="https://img.shields.io/github/actions/workflow/status/bltavares/axum-content-negotiation/main.yml?branch=master" />
</a>
<a href="https://github.com/bltavares/axum-content-negotiation/actions?query=workflow%3ACross-compile+branch%3Amaster">
    <img src="https://img.shields.io/github/actions/workflow/status/bltavares/axum-content-negotiation/main.yml?branch=master" />
</a>
<a href="https://crates.io/crates/axum-content-negotiation">
    <img src="https://img.shields.io/crates/v/axum-content-negotiation.svg" />
</a>
<a href="https://docs.rs/axum-content-negotiation">
    <img src="https://docs.rs/axum-content-negotiation/badge.svg" />
</a>
<hr />

A set of Axum Layers and Extractors that enable content negotiation using `Accept` and `Content-Type` headers.
It implements schemaless serialization and deserialization content negotiation. Currently supported encodings are:
- `application/json`
- `application/cbor`

## Installation

```toml
[dependencies]
axum-content-negotiation = "0.1"
```

### Features

The following features can be enabled to include support for different encodings:
- `simd-json` (default): Enables support for `application/json` encoding using `simd-json`.
- `cbor` (default): Enables support for `application/cbor` encoding using `cbor4ii`.
- `json`: Enables support for `application/json` encoding using `serde_json`.

The following features enable the default content type when `Accept` header is missing or `Accept: */*` is present:
- `default-json` (default): Assumes `application/json` as the default content type.
- `default-cbor`: Assumes `application/cbor` as the default content type.

In order to customize your dependencies, you can enable or disable the features as follows:

```toml
[dependencies]
axum-content-negotiation = { version = "0.1", default-features = false, features = ["json", "default-json"] }
```

## Usage

### Request payloads

The `axum_content_negotiation::Negotiate` is `Extractor` can be used in an Axum handlers to accept multiple `Content-Type` formats for the request body.
This extractor will attempt to deserialize the request body into the desired type based on the `Content-Type` header and a list of supported schemaless encodings.

```rust,no_run
use axum::{http::StatusCode, response::IntoResponse, routing::post, Router};
use axum_content_negotiation::Negotiate;

#[derive(serde::Deserialize, Debug)]
struct YourType {
    name: String,
}

async fn handler(Negotiate(request_body): Negotiate<YourType>) -> impl IntoResponse {
    (StatusCode::OK, format!("Received ${:?}", request_body))
}

let router: Router<()> = Router::new().route("/", post(handler));
```

### Response payloads

In order to respond with the correct `Content-Type` header, the `axum_content_negotiation::Negotiate` also implements an `IntoResponse` trait,
but it requires `axum_content_negotiation::NegotiateLayer` in order to actually perform the serialization on the desired format.

```rust,no_run
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_content_negotiation::{Negotiate, NegotiateLayer};

#[derive(serde::Serialize)]
struct YourType {
    name: String,
}

async fn handler() -> impl IntoResponse {
    let response = YourType {
        name: "John".to_string(),
    };
    (StatusCode::OK, Negotiate(response))
}

let router: Router<()> = Router::new().route("/", get(handler)).layer(NegotiateLayer);
```

## All together

```rust,no_run
use axum::{http::StatusCode, response::IntoResponse, routing::*, Router};
use axum_content_negotiation::{Negotiate, NegotiateLayer};

#[derive(serde::Deserialize, Debug)]
struct Input {
    name: String,
}

#[derive(serde::Serialize)]
struct Output {
    name: String,
}

async fn handler(Negotiate(request_body): Negotiate<Input>) -> impl IntoResponse {
    let response = Output {
        name: format!("Hello there, {}!", request_body.name),
    };
    (StatusCode::OK, Negotiate(response))
}

let router: Router<()> = Router::new().route("/", put(handler)).layer(NegotiateLayer);
```