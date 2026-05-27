test-nextest:
    cargo nextest run --workspace

test-nextest-no-fail-fast:
    cargo nextest run --workspace --no-fail-fast

test-ignored:
    cargo test --workspace -- --ignored

soroban-native-test:
    cargo test --workspace

soroban-wasm-smoke: wasm-build wasm-artifacts

test-ci: test-nextest soroban-wasm-smoke
