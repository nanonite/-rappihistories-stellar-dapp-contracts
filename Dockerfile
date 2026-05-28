FROM docker.io/nixos/nix:latest AS contracts
WORKDIR /workspace

ENV NIX_CONFIG="experimental-features = nix-command flakes"

COPY components/contracts components/contracts
WORKDIR /workspace/components/contracts

RUN rm -f .git
RUN nix flake check --no-write-lock-file --print-build-logs
RUN nix develop .#ci --command cargo test
RUN nix develop .#ci --command cargo build --release --target wasm32-unknown-unknown

FROM docker.io/nixos/nix:latest AS contract-artifacts
WORKDIR /workspace

ENV NIX_CONFIG="experimental-features = nix-command flakes"

COPY components/contracts components/contracts
WORKDIR /workspace/components/contracts

RUN rm -f .git
RUN nix develop .#ci --command cargo build --release --target wasm32-unknown-unknown

FROM docker.io/stellar/stellar-cli:26.1.0 AS contract-runner
WORKDIR /workspace

USER root
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
  && apt-get install -y --no-install-recommends bash ca-certificates libdbus-1-3 \
  && rm -rf /var/lib/apt/lists/*

RUN mkdir -p components/contracts/target/wasm32-unknown-unknown/release
COPY --from=contract-artifacts /workspace/components/contracts/target/wasm32-unknown-unknown/release/*.wasm components/contracts/target/wasm32-unknown-unknown/release/
COPY scripts/deploy-contracts.sh scripts/deploy-contracts.sh

ENV CONTRACTS_DIR=/workspace/components/contracts
ENV SKIP_CONTRACT_BUILD=1

ENTRYPOINT ["bash", "/workspace/scripts/deploy-contracts.sh"]
