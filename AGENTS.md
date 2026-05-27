# Agent Notes

- For identity-only WASM verification, use the Just shorthands:
  - `just identity-wasm-check`
  - `just identity-wasm-build`
- These recipes enter the project Nix toolchain and unset `RUSTC_WRAPPER`, avoiding repeated long commands and the local `sccache` permission failure.
