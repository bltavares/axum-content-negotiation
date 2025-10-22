
# Default action: check
default: check
.PHONY: default

# Runs cargo fmt and cargo clippy
lint:
	cargo fmt --all --check
	cargo clippy --all -- -D warnings
	cargo clippy --all --no-default-features --features json,default-json -- -D warnings
.PHONY: lint

# Fix lint issues when possible
lint-fix:
	cargo fmt --all
	cargo clippy --fix --all --allow-dirty --allow-staged -- -D warnings
	cargo clippy --fix --all --no-default-features --features json,default-json -- -D warnings
	cargo clippy --fix --all --no-default-features --features cbor,default-cbor -- -D warnings
.PHONY: lint-fix

# Be really annoying about lints
lint-pedantic:
	cargo clippy -- -D clippy::pedantic
	cargo clippy --no-default-features --features json,default-json -- -D clippy::pedantic
.PHONY: lint-pedantic

# Run cargo check for quick compilation instead of full build
check:
	cargo check --all
	cargo check --all --no-default-features --features json,default-json
.PHONY: check

# Run all tests
#
# uses cargo-run-bin to pin libs
test:
	cargo bin cargo-nextest run --all
	cargo bin cargo-nextest run --all --no-default-features --features json,default-json
	cargo bin cargo-nextest run --all --no-default-features --features cbor,default-cbor
.PHONY: test

# Build docs locally
doc:
	cargo doc --no-deps --all --open
.PHONY: doc

# Generate new entry for CHANGELOG.md
#
# uses cargo-run-bin to pin libs
changelog:
	cargo bin changelog -o CHANGELOG.md
.PHONY: changelog