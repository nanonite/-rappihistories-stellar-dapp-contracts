rust-build:
    cargo build --workspace

rust-release-build:
    cargo build --workspace --release

rust-check:
    cargo check --workspace

rust-release-check:
    cargo check --workspace --release

rust-test:
    cargo test --workspace

rust-ci: rust-check rust-test
