# Verification

Pre-ship and CI parity for this workspace. Full notes (lint policy, examples):
[`docs/VERIFICATION.md`](docs/VERIFICATION.md).

## Environment

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-pion
export PION_ALLOW_INSECURE=1
```

## Commands

Run from the repo root; each must pass with no warnings:

```bash
cargo fmt --all --check
cargo check -p pion --features runtime
cargo check -p pion-server
cargo check -p pion-bench
cargo clippy -p pion --all-targets --features runtime -- -D warnings
cargo clippy -p pion-server --all-targets -- -D warnings
cargo test -p pion --features runtime
cargo test -p pion-server
cargo test -p pion-e2e
cargo test -p pion-spectra-topics
RUSTDOCFLAGS="-D warnings" cargo doc -p pion --features runtime --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc -p pion-spectra-topics --no-deps
cargo test -p pion --doc --features runtime
cargo deny check
cargo llvm-cov -p pion --features runtime --fail-under-lines 50 --summary-only
```
