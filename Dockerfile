FROM rust:1.95.0-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/rushdb .
EXPOSE 6080

CMD ["./rushdb"]
