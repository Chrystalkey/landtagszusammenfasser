FROM oapi-preimage AS oapifile

FROM rust:1.85 AS builder

# RUN apt update && apt install -y --no-install-recommends pkg-config && rm -rf /var/lib/apt/lists/*

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
    cargo build -r

COPY ./.sqlx ./.sqlx
COPY ./src ./src
COPY ./migrations ./migrations
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && cargo build --release

FROM rust:1.85-slim-bookworm AS runner

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
            CMD curl -f "http://localhost:80" || exit 1


RUN apt update \
&&  apt install -y --no-install-recommends libssl-dev pkg-config libpq5 \
&&  rm -rf /var/lib/apt/lists/*

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder --chmod=0100 --chown=ltzf-database:ltzf-database /app/target/release/ltzf-db /app/ltzf-db

WORKDIR /app

USER ltzf-database

ENTRYPOINT ["./ltzf-db"]
