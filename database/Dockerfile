FROM rust:1.83 AS builder

RUN apt install -y libpq-dev

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./oapicode ./oapicode
COPY ./Cargo.lock ./Cargo.lock
COPY ./migrations ./migrations

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "ltzf-database"
    
RUN cargo build --release

RUN cp ./target/release/ltzusfas-db ltzf-db

RUN rm -rf target src migrations oapicode

#FROM debian:bullseye-slim

#RUN apt update && apt install -y libpq-dev openssl

#COPY --from=builder /etc/passwd /etc/passwd
#COPY --from=builder /etc/group /etc/group
#COPY --from=builder --chown=ltzf-database:ltzf-database /target/release/ltzusfas-db ./ltzf-database

USER ltzf-database

ENTRYPOINT ["./ltzf-db"]

