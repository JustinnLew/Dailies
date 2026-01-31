ARG RUST_VERSION=1.90.0

FROM rust:${RUST_VERSION} AS builder
WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim AS release

WORKDIR /app
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend .

ENV PORT=6666
EXPOSE 6666

CMD ["./backend"]

