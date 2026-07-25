FROM rust:alpine as builder
RUN apk add --no-cache musl-dev

WORKDIR /usr/src/outrrr
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache docker-cli ca-certificates tzdata

COPY --from=builder /usr/src/outrrr/target/release/outrrr /usr/local/bin/outrrr
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
COPY config.yaml /etc/outrrr/config.yaml

RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/outrrr

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
