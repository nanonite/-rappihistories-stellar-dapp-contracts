fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

wasm-clippy:
    cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings

wasm-quality: wasm-clippy wasm-check

quality: fmt-check clippy wasm-quality
