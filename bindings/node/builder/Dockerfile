# bindings/node/builder/Dockerfile

FROM node:20-bookworm

RUN apt-get update && apt-get install -y \
    curl wget xz-utils build-essential pkg-config \
    gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

RUN wget https://musl.cc/aarch64-linux-musl-cross.tgz -O /tmp/aarch64-linux-musl-cross.tgz \
    && tar -xf /tmp/aarch64-linux-musl-cross.tgz -C /opt \
    && rm /tmp/aarch64-linux-musl-cross.tgz
ENV PATH="/opt/aarch64-linux-musl-cross/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    aarch64-unknown-linux-musl

RUN mkdir -p /root/.cargo && \
    echo "[target.aarch64-unknown-linux-gnu]\nlinker = \"aarch64-linux-gnu-gcc\"\n\n[target.aarch64-unknown-linux-musl]\nlinker = \"aarch64-linux-musl-gcc\"" > /root/.cargo/config.toml

RUN npm install -g @napi-rs/cli@latest

WORKDIR /workspace