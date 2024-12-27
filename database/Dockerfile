FROM oapi-preimage AS oapifile

FROM rust:1.83 AS builder

RUN apt update && apt install -y --no-install-recommends libpq-dev && rm -rf /var/lib/apt/lists/*

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

FROM debian:bookworm-slim AS runner

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
            CMD curl -f "http://localhost:80" || exit 1


RUN apt update \
&&  apt install -y --no-install-recommends libpq5 \
&&  rm -rf /var/lib/apt/lists/*

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder --chmod=+x --chown=ltzf-database:ltzf-database /app/target/release/ltzusfas-db /app/ltzusfas-db

WORKDIR /app

USER ltzf-database
ENTRYPOINT ["/app/ltzusfas-db"]
