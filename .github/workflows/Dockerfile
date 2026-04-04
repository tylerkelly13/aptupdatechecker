FROM rust:slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libapt-pkg-dev \
    libdbus-1-dev \
    pkg-config \
    curl \
    libssl-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-deb cargo-tarpaulin just \
    && rm -rf /usr/local/cargo/registry

RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin
