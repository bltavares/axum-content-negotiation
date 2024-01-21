
# Default action: check
default: check
.PHONY: default

# Runs cargo fmt and cargo clippy
lint:
	cargo fmt --all --check
	cargo clippy --all -- -D warnings
	cargo clippy --all --no-default-features --features json -- -D warnings
.PHONY: lint

# Fix lint issues when possible
lint-fix:
	cargo fmt --all
	cargo clippy --fix --all --allow-dirty --allow-staged -- -D warnings
	cargo clippy --fix --all --no-default-features --features json -- -D warnings
.PHONY: lint-fix

# Be really annoying about lints
lint-pedantic:
	cargo clippy -- -D clippy::pedantic
	cargo clippy --no-default-features --features json -- -D clippy::pedantic
.PHONY: lint-pedantic

# Run cargo check for quick compilation instead of full build
check:
	cargo check --all
	cargo check --all --no-default-features --features json
.PHONY: check

# Run all tests
test:
	cargo test --all
	cargo test --all --no-default-features --features json
.PHONY: test

# Build docs locally
doc:
	cargo doc --no-deps --all --open
.PHONY: doc