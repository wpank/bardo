# Build all crates (debug)
build:
	cargo build --workspace

# Build release binary
build-release:
	cargo build --release -p golem-binary

# Run all tests with nextest
test:
	cargo nextest run --workspace

# Run tests in CI mode (fail-fast)
test-ci:
	cargo nextest run --workspace --profile ci

# Run clippy on all crates
lint:
	cargo clippy --workspace --all-features -- -D warnings

# Format all crates in-place
fmt:
	cargo fmt --all

# Check formatting (CI)
fmt-check:
	cargo fmt --all -- --check

# Run cargo-deny checks (license + advisory)
deny:
	cargo deny check

# Generate coverage report
coverage:
	cargo llvm-cov nextest --workspace --html

# Build docs
docs:
	cargo doc --workspace --no-deps --open

# Build mdbook docs
mdbook:
	cd docs && mdbook build

# Watch mode (requires bacon)
watch:
	bacon clippy

# Release builds for deployment targets
release-linux-amd64:
	cargo build --release --target x86_64-unknown-linux-musl -p golem-binary

release-linux-arm64:
	cargo build --release --target aarch64-unknown-linux-musl -p golem-binary

# Run mirage-rs dev fork
mirage rpc_url="":
	cargo run -p mirage-rs -- --rpc-url {{rpc_url}}

# Full CI check sequence
ci: fmt-check lint test deny
	@echo "CI passed"
