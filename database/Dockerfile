FROM rust:1.83 AS builder

RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "ltzf-database"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY ./src ./src
COPY ./oapicode ./oapicode
COPY ./migrations ./migrations

RUN cargo build --release && \
    cp ./target/release/ltzusfas-db /app/ltzf-db

FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y libpq-dev openssl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

COPY --from=builder --chown=ltzf-database:ltzf-database /app/ltzf-db ./

USER ltzf-database

ENTRYPOINT ["./ltzf-db"]

