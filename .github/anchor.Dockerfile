# Check https://hub.docker.com/layers/solanafoundation/anchor/v0.31.1/images/
# and https://github.com/solana-foundation/anchor/blob/master/docker/build/Dockerfile for build environment details.
#
# Ubuntu: 22 -> 24.04 due to LiteSVM contraints.
# SOLANA_CLI: 2.1.0 -> 3.0.6
# ANCHOR_CLI: 0.32.1
# RUSTUP_TOOLCHAIN_VERSION: 1.91.1
# NODE_VERSION: 20.16.0 -> 24.11.1
# others: installed zip
#
# e.g. docker build . -f ./.github/anchor.Dockerfile --platform=linux/amd64 -t ghcr.io/fragmetric-labs/solana-anchor:v0.32.1

FROM ubuntu:24.04@sha256:4fdf0125919d24aec972544669dcd7d6a26a8ad7e6561c73d5549bd6db258ac2

LABEL org.opencontainers.image.source="https://github.com/fragmetric-labs/fragmetric-contracts"
LABEL org.opencontainers.image.description="Verifiable builder image for Anchor based Solana programs"

ARG DEBIAN_FRONTEND=noninteractive

ARG SOLANA_CLI="3.0.6"
ARG ANCHOR_CLI="0.32.1"
ARG NODE_VERSION="24.11.1"

ENV HOME="/root"
ENV PATH="${HOME}/.cargo/bin:${PATH}"
ENV PATH="${HOME}/.local/share/solana/install/active_release/bin:${PATH}"
ENV PATH="${HOME}/.nvm/versions/node/v${NODE_VERSION}/bin:${PATH}"

# Install base utilities.
RUN mkdir -p /workdir && mkdir -p /tmp && \
    apt-get update -qq && apt-get upgrade -qq && apt-get install -qq \
    build-essential git curl wget jq pkg-config python3-pip \
    libssl-dev libudev-dev

# Install rust.
RUN curl "https://sh.rustup.rs" -sfo rustup.sh && \
    sh rustup.sh -y && \
    rustup component add rustfmt clippy

# Install node / npm / yarn / pnpm.
RUN curl -o- https://raw.githubusercontent.com/creationix/nvm/v0.33.11/install.sh | bash
ENV NVM_DIR="${HOME}/.nvm"
RUN . $NVM_DIR/nvm.sh && \
    nvm install v${NODE_VERSION} && \
    nvm use v${NODE_VERSION} && \
    nvm alias default node && \
    npm install -g yarn pnpm

# Install Solana tools.
RUN sh -c "$(curl -sSfL https://release.anza.xyz/v${SOLANA_CLI}/install)"

# Install anchor.
RUN cargo install --git https://github.com/coral-xyz/anchor --tag v${ANCHOR_CLI} anchor-cli --locked

# Build a dummy program to bootstrap the BPF SDK (doing this speeds up builds).
RUN mkdir -p /tmp && cd tmp && anchor init dummy && cd dummy && anchor build

# Install other tools
RUN apt-get install -qq zip

WORKDIR /workdir