ARG RUST_VERSION=1.90.0

FROM rust:${RUST_VERSION}
WORKDIR /app
COPY . .

RUN cargo build --release

ENV PORT=3000
EXPOSE 3000

CMD ["./target/release/backend"]

