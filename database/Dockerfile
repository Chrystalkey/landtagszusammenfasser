FROM rust:1.83 AS builder

RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates && openssl


COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./oapicode ./oapicode
COPY ./Cargo.lock ./Cargo.lock

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "ltzf-database"

RUN cargo build --release --target x86_64-unknown-linux-musl


FROM scratch


COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder --chown=ltzf-database:ltzf-databaes /target/x86_64-unknown-linux-musl/release/database ./ltzf-database

USER ltzf-database

ENTRYPOINT ["./ltzf-database"]

