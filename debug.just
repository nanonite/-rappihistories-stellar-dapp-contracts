rust-debug-build:
    cargo build --workspace

rust-debug-check:
    cargo check --workspace

wasm-debug-build:
    cargo build --workspace --target wasm32-unknown-unknown

wasm-debug-check:
    cargo check --workspace --target wasm32-unknown-unknown

debug-build: rust-debug-build wasm-debug-build

debug-check: rust-debug-check wasm-debug-check

debug-ci: debug-check debug-build
