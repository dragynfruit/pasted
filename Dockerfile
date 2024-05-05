FROM rust:1.77.2-bookworm as builder

WORKDIR /usr/src/pasted
COPY . .

RUN cargo install --path .
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/pasted/target/release/pasted /usr/local/bin/pasted/pasted
COPY --from=builder /usr/src/pasted/templates /usr/local/bin/pasted/templates
RUN chmod +x /usr/local/bin/pasted/pasted

WORKDIR /usr/local/bin/
CMD ["./pasted"]
