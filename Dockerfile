FROM rust:alpine AS builder

WORKDIR /usr/src/pasted
COPY . .

RUN apk add --no-cache -U musl-dev openssl-dev
ENV OPENSSL_DIR=/usr
RUN cargo build --release

FROM alpine:latest

COPY --from=builder /usr/src/pasted/target/release/pasted /usr/local/bin/pasted/pasted
COPY --from=builder /usr/src/pasted/templates /usr/local/bin/pasted/templates

WORKDIR /usr/local/bin/pasted
CMD ["./pasted"]
