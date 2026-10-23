# regit-daycount — Task runner
# Run `just` to see available recipes.

# Quality gate — run all checks
check: fmt-check lint test test-min doc
    @echo "All checks passed."

# Format check
fmt-check:
    cargo fmt --all --check

# Format (fix)
fmt:
    cargo fmt --all

# Lint — zero warnings, all targets, all features
lint:
    cargo clippy --all-targets --all-features --locked -- -D warnings

# Run all tests (default features)
test:
    cargo test --locked

# Run tests with no default features (calendar tables disabled)
test-min:
    cargo test --no-default-features --locked

# Build documentation
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked

# Run the library quickstart example
example:
    cargo run --example quickstart

# Run benchmarks
bench:
    cargo bench

# Dependency / licence audit (requires cargo-deny)
deny:
    cargo deny check

# WASM build smoke test
wasm:
    cargo build --target wasm32-unknown-unknown --release --locked

# Genuine no_std build — a target with no `std` at all; proves the crate is no_std
nostd:
    cargo build --target thumbv7em-none-eabi --no-default-features --locked
    cargo build --target thumbv7em-none-eabi --locked

# Run property tests with extra cases
proptest:
    PROPTEST_CASES=5000 cargo test prop_

# Full CI pipeline
ci: fmt-check lint test test-min doc deny wasm nostd msrv oracle package

# Run Miri for undefined-behaviour checks
miri:
    cargo +nightly miri test --lib

# Recompute pinned Python oracle fixtures and compare the current Rust crate
oracle:
    uv run validation/oracle.py --check --rust

# Regenerate expected results with the pinned Python reference libraries
oracle-regen:
    uv run validation/oracle.py

# Verify the published library's minimum Rust version, without development targets
msrv:
    cargo +1.85.0 build --lib --all-features --locked
    cargo +1.85.0 build --lib --no-default-features --locked

# Verify the exact archive that will be uploaded (requires a clean checkout)
package:
    cargo package --locked
