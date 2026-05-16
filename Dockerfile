FROM rust:1.95.0-slim AS builder

WORKDIR /app
COPY . .

RUN cargo build --release -p cli
RUN cargo build --release -p db
RUN cargo build --release -p benchmark

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/db .
COPY --from=builder /app/target/release/cli /usr/local/bin/cli
COPY --from=builder /app/target/release/benchmark /usr/local/bin/benchmark

EXPOSE 6080

CMD ["./db"]
