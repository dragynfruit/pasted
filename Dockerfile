FROM rust:1.77.2-bookworm as builder

WORKDIR /usr/src/pasted
COPY . .

RUN cargo install --path .
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/pasted/target/release/pasted /usr/local/bin/pasted

WORKDIR /usr/local/bin/
CMD ["./pasted"]
