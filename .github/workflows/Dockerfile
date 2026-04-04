FROM rust:slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libapt-pkg-dev \
    libdbus-1-dev \
    pkg-config \
    curl \
    libssl-dev \
    zlib1g-dev \
    g++ \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt \
    && cargo install cargo-deb cargo-tarpaulin \
    && rm -rf /usr/local/cargo/registry

COPY --from=ghcr.io/casey/just:latest /just /usr/local/bin/
