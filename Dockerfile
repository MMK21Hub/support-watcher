FROM rust:1.88-bullseye AS builder

WORKDIR /app
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.lock ./

RUN cargo build --release

FROM debian:bullseye-slim AS runner

WORKDIR /app
COPY --from=builder /app/target/release/support-watcher /app/support-watcher

# Install CA certificates so that we can do HTTPS
RUN apt-get update && apt-get install -y ca-certificates

ENTRYPOINT ["/app/support-watcher"]