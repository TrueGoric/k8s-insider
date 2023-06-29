ARG RUST_VERSION=1-bookworm

FROM rust:${RUST_VERSION} as app-build

WORKDIR /app
COPY /app /app

RUN --mount=type=cache,target=/var/cache/apt \
        apt update \
        && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN --mount=type=cache,target=/app/target \
        cargo build \
        --target x86_64-unknown-linux-musl \
        --package k8s-insider-agent \
        --release \
        && mkdir -p /out \
        && cp /app/target/x86_64-unknown-linux-musl/release/k8s-insider-agent /out/k8s-insider-agent
