# Local public-contract gate.
check:
    cargo fmt --all --check
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo test --locked
    cargo test --no-default-features --locked
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --no-default-features --locked

# Independently execute Python and Rust, compare observations and record coverage.
validate:
    uv run tests/run.py --coverage

# cargo-semver-checks 0.48 consumes the Rust 1.95 rustdoc format.
# Both tool versions are pinned in CI/release; newer crate builds use rust-toolchain.toml.
semver:
    cargo +1.95.0 semver-checks --all-features --baseline-rev "$(git describe --tags --abbrev=0)"

msrv:
    cargo +1.85.0 build --lib --all-features --locked
    cargo +1.85.0 build --lib --no-default-features --locked

platforms:
    cargo build --target wasm32-unknown-unknown --release --locked
    cargo build --target thumbv7em-none-eabi --no-default-features --locked
    cargo build --target thumbv7em-none-eabi --locked

deny:
    cargo deny check

package:
    cargo package --locked
