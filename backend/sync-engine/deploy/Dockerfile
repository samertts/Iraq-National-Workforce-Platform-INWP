FROM rust:1.96-slim-bookworm AS builder

RUN apt-get update && apt-get install -y protobuf-compiler pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
RUN cargo build --release --bin sync-engine

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sync-engine /usr/local/bin/sync-engine

EXPOSE 50052 50053 8080 8081 9090 8082

ENTRYPOINT ["/usr/local/bin/sync-engine"]
