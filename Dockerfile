FROM rust:alpine AS builder

WORKDIR /usr/src/pasted
COPY . .

RUN apk add --no-cache -U musl-dev openssl-dev
ENV OPENSSL_DIR=/usr
RUN cargo build --release --features include_templates

FROM alpine:latest

COPY --from=builder /usr/src/pasted/target/release/pasted /usr/local/bin/pasted/pasted

WORKDIR /usr/local/bin/pasted
CMD ["./pasted"]
