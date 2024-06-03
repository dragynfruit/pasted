FROM rust:1.80.0-alpine as builder

WORKDIR /usr/src/pasted
COPY . .

RUN cargo build --release

FROM alpine:3.20.0

COPY --from=builder /usr/src/pasted/target/release/pasted /usr/local/bin/pasted/pasted
COPY --from=builder /usr/src/pasted/templates /usr/local/bin/pasted/templates

WORKDIR /usr/local/bin/pasted
CMD ["./pasted"]
