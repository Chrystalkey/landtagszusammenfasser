FROM oapi-preimage AS oapifile

FROM rust:1.83 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends libpq-dev && rm -rf /var/lib/apt/lists/*

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
COPY --from=oapifile /app/oapicode-rust ./oapicode

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY ./src ./src
COPY ./migrations ./migrations

RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libpq-dev libssl-dev && \
    rm -rf /var/lib/apt/lists/*

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
CMD curl -f "http://localhost:80" || exit 1

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

COPY --from=builder --chown=ltzf-database:ltzf-database /app/target/release/ltzusfas-db ./

RUN chmod +x ./ltzusfas-db

USER ltzf-database

ENTRYPOINT ["/app/ltzusfas-db"]
