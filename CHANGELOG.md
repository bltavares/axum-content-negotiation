## 2025-07-02, Version v0.2.0
### Commits
- [[`9eab886257`](https://github.com/vipul43/axum-content-negotiation/commit/9eab886257dacc70cd56ac5db3804f2c48741959)] feat: add support for `application/x-www-urlencoded` content type & update depedencies (vipul43)
- [[`03ec8232cd`](https://github.com/vipul43/axum-content-negotiation/commit/03ec8232cd5af1a267ea30b31fd78f278726e15f)] chore: Update the changelog (vipul43)

## 2025-05-16, Version v0.1.3
### Commits
- [[`e6eca94052`](https://github.com/bltavares/axum-content-negotiation/commit/e6eca940521d44df9dfc799f0934fdd291c88615)] chore: Release axum-content-negotiation version 0.1.3 (Bruno Tavares)
- [[`29117d515d`](https://github.com/bltavares/axum-content-negotiation/commit/29117d515df83893dd2fd306b830eae51e1326ea)] Merge pull request #6 from bltavares/optmize-multiple-q-entries (Bruno Tavares)
- [[`fa78b21684`](https://github.com/bltavares/axum-content-negotiation/commit/fa78b2168419e047f0e13d8b37ec7aeb72b9899a)] Optmize the handling of multiple encoding formats (Bruno Tavares)
- [[`c1d44f0014`](https://github.com/bltavares/axum-content-negotiation/commit/c1d44f00146fa7f3cf5e406872d27757effde519)] Merge pull request #5 from notNotDaniel/accept-multiple-mime-types (Bruno Tavares)
- [[`c01536d73c`](https://github.com/bltavares/axum-content-negotiation/commit/c01536d73c45b9addc8ba6afad167c96aca6ba66)] run cargo fmt (Daniel Keller)
- [[`a9848e163d`](https://github.com/bltavares/axum-content-negotiation/commit/a9848e163d64cb2a8d5eb2d7403ea49b93ff71e0)] remove unneeded return statement, to satisfy clippy (Daniel Keller)
- [[`0d1b154520`](https://github.com/bltavares/axum-content-negotiation/commit/0d1b1545201ed28cf40a5140b8f287772e13ad00)] select the correct mime type given equal q values (Daniel Keller)
- [[`d931cffa72`](https://github.com/bltavares/axum-content-negotiation/commit/d931cffa72ea2ca372dc7127e7cff28e50a0f4b6)] support multiple mime-types in the Accept header, and honor q= values (Daniel Keller)
- [[`7be677598b`](https://github.com/bltavares/axum-content-negotiation/commit/7be677598b81de24c111b6adf0d8dcadc5f10321)] Update the changelog (Bruno Tavares)

### Stats
```diff
 CHANGELOG.md |  18 +++-
 Cargo.lock   |   2 +-
 Cargo.toml   |   2 +-
 src/lib.rs   | 386 ++++++++++++++++++++++++++++++++++++++++++++++++++++--------
 4 files changed, 356 insertions(+), 52 deletions(-)
```


## 2025-01-05, Version v0.1.2
### Commits
- [[`8a91d5b6e6`](https://github.com/bltavares/axum-content-negotiation/commit/8a91d5b6e6237bb8037cf7a1f1da973368ec7c56)] chore: Release axum-content-negotiation version 0.1.2 (Bruno Tavares)
- [[`bc151c3f71`](https://github.com/bltavares/axum-content-negotiation/commit/bc151c3f716fbb30722c7d10e02d4195251b02e2)] dev: Include dependency on semver checks for dev tasks (Bruno Tavares)
- [[`724886ce56`](https://github.com/bltavares/axum-content-negotiation/commit/724886ce56610a7c813e0d36408f1460c5e67d56)] Merge pull request #4 from bltavares/upgrade-deps (Bruno Tavares)
- [[`951b3ae8da`](https://github.com/bltavares/axum-content-negotiation/commit/951b3ae8dae00958d99ccc7bc0a5004b7d117938)] Upgrade axum to 0.8.x (Bruno Tavares)
- [[`4352aa5098`](https://github.com/bltavares/axum-content-negotiation/commit/4352aa509899d711f3aac875786459987133fb7c)] Update the changelog (Bruno Tavares)

### Stats
```diff
 CHANGELOG.md |  34 ++++++++-
 Cargo.lock   | 252 ++++++++----------------------------------------------------
 Cargo.toml   |  12 +--
 src/lib.rs   |  14 +--
 4 files changed, 85 insertions(+), 227 deletions(-)
```


## 2024-04-27, Version v0.1.1
### Commits
- [[`04cc447c30`](https://github.com/bltavares/axum-content-negotiation/commit/04cc447c30a74e552f31723f6a9845aa8e4251f6)] chore: Release axum-content-negotiation version 0.1.1 (Bruno Tavares)
- [[`4c8aa1eaa5`](https://github.com/bltavares/axum-content-negotiation/commit/4c8aa1eaa5e19e22df38d98f0d941c88f220799f)] Merge pull request #2 from jbourassa/reset-content-length (Bruno Tavares)
- [[`fcdbb2f365`](https://github.com/bltavares/axum-content-negotiation/commit/fcdbb2f36591a0ec55602dd9edc7f2f8677c7f36)] Reset content length (Jimmy Bourassa)
- [[`d88bb45a5c`](https://github.com/bltavares/axum-content-negotiation/commit/d88bb45a5cd9becd834efcec753ce5a428bc0bb5)] Rename variable into a more meaninful name (leftover of lint fixes) (Bruno Tavares)
- [[`3082caf72c`](https://github.com/bltavares/axum-content-negotiation/commit/3082caf72cb4df2ecc05573a90f491f53f4172e9)] Fix branch name on README (Bruno Tavares)
- [[`876826e37d`](https://github.com/bltavares/axum-content-negotiation/commit/876826e37da5b69d7b65089294ceedb20ae7df05)] Merge branch 'Testing' (Bruno Tavares)
- [[`b39117530b`](https://github.com/bltavares/axum-content-negotiation/commit/b39117530b5725b1d7913aa661ced045c441fa13)] README (Bruno Tavares)
- [[`9307225220`](https://github.com/bltavares/axum-content-negotiation/commit/9307225220617a4268f25787c9ab845143b41d0a)] ARMv7 does not have simd (Bruno Tavares)
- [[`e7505419c5`](https://github.com/bltavares/axum-content-negotiation/commit/e7505419c5090619ebc83057308ca541451d3828)] Remove MIPs from CI as it's not stable anymore (Bruno Tavares)
- [[`fb7a940e97`](https://github.com/bltavares/axum-content-negotiation/commit/fb7a940e9726a6546582d0c11a428426153fbcbc)] Initial GH Actions setup (Bruno Tavares)
- [[`55238224b1`](https://github.com/bltavares/axum-content-negotiation/commit/55238224b19977f1a2894a67a6e5a29ad70d0839)] Documentation (Bruno Tavares)
- [[`cd83102e32`](https://github.com/bltavares/axum-content-negotiation/commit/cd83102e32e3be714e00b7a193e0ad4cf1c140e5)] Preparation for release (Bruno Tavares)
- [[`730c0dec4c`](https://github.com/bltavares/axum-content-negotiation/commit/730c0dec4ccdc44316a28dce65def0b4831abf28)] More tests (Bruno Tavares)
- [[`63acdfba38`](https://github.com/bltavares/axum-content-negotiation/commit/63acdfba38f4e75efdb049ca3e369bcd414f3326)] Initial commit (Bruno Tavares)

### Stats
```diff
 .github/workflows/cross_compile.yml |   30 +-
 .github/workflows/main.yml          |   87 +++-
 .gitignore                          |    2 +-
 Cargo.lock                          | 1145 ++++++++++++++++++++++++++++++++++++-
 Cargo.toml                          |   45 +-
 LICENSE-APACHE                      |  201 ++++++-
 LICENSE-MIT                         |   21 +-
 Makefile                            |   48 ++-
 README.md                           |  120 ++++-
 bacon.toml                          |   81 +++-
 src/lib.rs                          |  940 ++++++++++++++++++++++++++++++-
 11 files changed, 2720 insertions(+)
```


