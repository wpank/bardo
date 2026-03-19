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

# Install all dev tools (uses binstall for pre-built binaries when available)
setup:
	cargo install cargo-binstall 2>/dev/null || true
	cargo binstall -y cargo-nextest cargo-deny cargo-llvm-cov bacon cargo-machete cargo-semver-checks taplo-cli 2>/dev/null || \
		cargo install cargo-nextest@0.9.100 --locked cargo-deny cargo-llvm-cov bacon
	brew install sccache lld
	@echo "Dev tools installed."

# Pre-compile workspace (warm sccache + incremental caches)
warm:
	cargo check --workspace
	cargo nextest run --workspace --no-run 2>/dev/null || cargo test --workspace --no-run
	@echo "Workspace warm."

# Build timing report (opens in browser)
timings:
	cargo build --workspace --timings
	open target/cargo-timings/cargo-timing.html

# Show which crates have changed (useful for scoped checks)
changed:
	@echo "Changed .rs files:"; git diff --name-only HEAD -- '*.rs' 2>/dev/null; git diff --name-only --cached -- '*.rs' 2>/dev/null

# Nextest archive: build test binaries once, replay anywhere
test-archive:
	cargo nextest archive --workspace --archive-file target/nextest-archive.tar.zst
	@echo "Archive: target/nextest-archive.tar.zst"

# Run tests from a pre-built archive (no recompilation)
test-replay:
	cargo nextest run --archive-file target/nextest-archive.tar.zst

# Install git hooks
hooks:
	git config core.hooksPath .githooks
	@echo "Git hooks installed."

# Auto-apply clippy suggestions
fix:
	cargo clippy --workspace --all-features --fix --allow-dirty

# Run with debug logging
run-debug *args:
	RUST_LOG=debug cargo run -p golem-binary -- {{args}}

# Trace a specific crate
trace crate *args:
	RUST_LOG={{crate}}=trace cargo run -p golem-binary -- {{args}}

# Find unused dependencies
unused-deps:
	cargo machete

# Check for semver violations across workspace crates
semver:
	cargo semver-checks check-release --workspace

# Lint all TOML files for consistency
toml-check:
	taplo check

# Show duplicate dependency versions
dupes:
	cargo tree --workspace --duplicates

# Deep audit: all quality gates
audit: fmt-check lint deny unused-deps toml-check
	@echo "Audit passed."

# Mutation testing on changed code (against main)
mutants:
	cargo mutants --in-diff <(git diff main) --test-tool=nextest
