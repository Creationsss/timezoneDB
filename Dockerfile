# Stage 1: Build
FROM rustlang/rust:nightly AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY public ./public

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/timezone-db /usr/local/bin/app
COPY --from=builder /app/public ./public

ENV RUST_LOG=info

CMD ["/usr/local/bin/app"]
