FROM rust:1.94-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY ipo-backend-shared/Cargo.toml ipo-backend-shared/Cargo.toml
COPY ipo-api/Cargo.toml ipo-api/Cargo.toml
COPY ipo-info-fetcher/Cargo.toml ipo-info-fetcher/Cargo.toml
COPY ipo-result-checker/Cargo.toml ipo-result-checker/Cargo.toml

RUN mkdir -p ipo-backend-shared/src && echo "" > ipo-backend-shared/src/lib.rs \
    && mkdir -p ipo-api/src && echo "fn main() {}" > ipo-api/src/main.rs \
    && mkdir -p ipo-info-fetcher/src && echo "fn main() {}" > ipo-info-fetcher/src/main.rs \
    && mkdir -p ipo-result-checker/src && echo "fn main() {}" > ipo-result-checker/src/main.rs
RUN cargo build --release --package ipo-info-fetcher

COPY ipo-backend-shared/src ipo-backend-shared/src
COPY ipo-info-fetcher/src ipo-info-fetcher/src
RUN touch ipo-backend-shared/src/lib.rs ipo-info-fetcher/src/main.rs \
    && cargo build --release --package ipo-info-fetcher

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ipo-info-fetcher /usr/local/bin/
ENV PORT=8082
EXPOSE 8082
CMD ["ipo-info-fetcher"]
