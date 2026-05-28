# syntax=docker/dockerfile:1.7

ARG RUST_IMAGE=docker.io/library/rust:1.83-slim-bookworm

FROM ${RUST_IMAGE} AS contract-deps
WORKDIR /workspace/components/contracts

ENV CARGO_HOME=/root/.cargo

RUN apt-get update \
  && apt-get install -y --no-install-recommends binaryen ca-certificates pkg-config \
  && rm -rf /var/lib/apt/lists/* \
  && rustup target add wasm32-unknown-unknown

RUN mkdir -p access-broker identity incentive medical-record prescription supplychain

COPY components/contracts/Cargo.toml components/contracts/Cargo.lock ./
COPY components/contracts/access-broker/Cargo.toml access-broker/Cargo.toml
COPY components/contracts/identity/Cargo.toml identity/Cargo.toml
COPY components/contracts/incentive/Cargo.toml incentive/Cargo.toml
COPY components/contracts/medical-record/Cargo.toml medical-record/Cargo.toml
COPY components/contracts/prescription/Cargo.toml prescription/Cargo.toml
COPY components/contracts/supplychain/Cargo.toml supplychain/Cargo.toml

RUN for crate in access-broker identity incentive medical-record prescription supplychain; do \
    mkdir -p "$crate/src"; \
    printf '%s\n' '#![no_std]' > "$crate/src/lib.rs"; \
  done
RUN --mount=type=cache,id=medichain-contracts-cargo-registry,target=/root/.cargo/registry \
  --mount=type=cache,id=medichain-contracts-cargo-git,target=/root/.cargo/git \
  cargo fetch --locked --target wasm32-unknown-unknown
RUN --mount=type=cache,id=medichain-contracts-cargo-registry,target=/root/.cargo/registry \
  --mount=type=cache,id=medichain-contracts-cargo-git,target=/root/.cargo/git \
  cargo fetch --locked

FROM contract-deps AS contracts
WORKDIR /workspace/components/contracts

COPY components/contracts/Cargo.toml components/contracts/Cargo.lock ./
COPY components/contracts/access-broker/src access-broker/src
COPY components/contracts/identity/src identity/src
COPY components/contracts/incentive/src incentive/src
COPY components/contracts/medical-record/src medical-record/src
COPY components/contracts/prescription/src prescription/src
COPY components/contracts/supplychain/src supplychain/src

RUN --mount=type=cache,id=medichain-contracts-cargo-registry,target=/root/.cargo/registry \
  --mount=type=cache,id=medichain-contracts-cargo-git,target=/root/.cargo/git \
  --mount=type=cache,id=medichain-contracts-target,target=/workspace/components/contracts/target \
  cargo test --workspace
RUN --mount=type=cache,id=medichain-contracts-cargo-registry,target=/root/.cargo/registry \
  --mount=type=cache,id=medichain-contracts-cargo-git,target=/root/.cargo/git \
  --mount=type=cache,id=medichain-contracts-target,target=/workspace/components/contracts/target \
  cargo build --workspace --release --target wasm32-unknown-unknown

FROM contract-deps AS contract-artifacts
WORKDIR /workspace/components/contracts

COPY components/contracts/Cargo.toml components/contracts/Cargo.lock ./
COPY components/contracts/access-broker/src access-broker/src
COPY components/contracts/identity/src identity/src
COPY components/contracts/incentive/src incentive/src
COPY components/contracts/medical-record/src medical-record/src
COPY components/contracts/prescription/src prescription/src
COPY components/contracts/supplychain/src supplychain/src

RUN --mount=type=cache,id=medichain-contracts-cargo-registry,target=/root/.cargo/registry \
  --mount=type=cache,id=medichain-contracts-cargo-git,target=/root/.cargo/git \
  --mount=type=cache,id=medichain-contracts-target,target=/workspace/components/contracts/target \
  cargo build --workspace --release --target wasm32-unknown-unknown \
  && mkdir -p /workspace/contract-artifacts \
  && cp target/wasm32-unknown-unknown/release/*.wasm /workspace/contract-artifacts/

FROM docker.io/stellar/stellar-cli:26.1.0 AS contract-runner
WORKDIR /workspace

USER root
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
  && apt-get install -y --no-install-recommends bash ca-certificates libdbus-1-3 \
  && rm -rf /var/lib/apt/lists/*

RUN mkdir -p components/contracts/target/wasm32-unknown-unknown/release
COPY --from=contract-artifacts /workspace/contract-artifacts/*.wasm components/contracts/target/wasm32-unknown-unknown/release/
COPY scripts/deploy-contracts.sh scripts/deploy-contracts.sh

ENV CONTRACTS_DIR=/workspace/components/contracts
ENV SKIP_CONTRACT_BUILD=1

ENTRYPOINT ["bash", "/workspace/scripts/deploy-contracts.sh"]
