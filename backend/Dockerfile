ARG RUST_VERSION=1.90.0
FROM rust:${RUST_VERSION}-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /app/backend

ENV PORT=3000
EXPOSE 3000

CMD ["./backend"]