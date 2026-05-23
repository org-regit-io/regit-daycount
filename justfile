# regit-daycount — Task runner
# Run `just` to see available recipes.

# Quality gate — run all checks
check: fmt-check lint test doc
    @echo "All checks passed."

# Format check
fmt-check:
    cargo fmt --all --check

# Format (fix)
fmt:
    cargo fmt --all

# Lint — zero warnings, all targets, all features
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests (default features)
test:
    cargo test

# Run tests with no default features (calendar tables disabled)
test-min:
    cargo test --no-default-features

# Build documentation
doc:
    cargo doc --no-deps --all-features

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
    cargo build --target wasm32-unknown-unknown --release

# Genuine no_std build — a target with no `std` at all; proves the crate is no_std
nostd:
    cargo build --target thumbv7em-none-eabi --no-default-features
    cargo build --target thumbv7em-none-eabi

# Run property tests with extra cases
proptest:
    PROPTEST_CASES=5000 cargo test prop_

# Full CI pipeline
ci: fmt-check lint test test-min doc deny wasm nostd

# Run Miri for undefined-behaviour checks
miri:
    cargo +nightly miri test --lib
