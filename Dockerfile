FROM rust:1.88-bullseye AS builder

WORKDIR /app
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.lock ./

RUN cargo build --release

FROM debian:bullseye-slim AS runner

WORKDIR /app
COPY --from=builder /app/target/release/support-watcher /app/support-watcher

# Add Tini
ENV TINI_VERSION=v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini
ENTRYPOINT ["/tini", "--"]
# Install CA certificates so that we can do HTTPS
RUN apt-get update && apt-get install -y ca-certificates

CMD ["/app/support-watcher"]