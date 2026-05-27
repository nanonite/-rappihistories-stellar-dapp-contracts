# syntax=docker/dockerfile:1.7

FROM docker.io/nixos/nix:latest AS contracts
WORKDIR /workspace

ENV NIX_CONFIG="experimental-features = nix-command flakes"

COPY components/contracts components/contracts
WORKDIR /workspace/components/contracts

RUN nix flake check --no-write-lock-file --print-build-logs
RUN nix develop .#ci --command cargo test
RUN nix develop .#ci --command cargo build --release --target wasm32-unknown-unknown
