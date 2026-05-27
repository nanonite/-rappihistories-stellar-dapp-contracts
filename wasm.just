wasm-build: wasm-release-build

wasm-release-build:
    cargo build --workspace --release --target wasm32-unknown-unknown

wasm-check: wasm-release-check

wasm-release-check:
    cargo check --workspace --release --target wasm32-unknown-unknown

wasm-artifacts:
    @ls -1 target/wasm32-unknown-unknown/release/*.wasm

wasm-ci: wasm-check wasm-build
