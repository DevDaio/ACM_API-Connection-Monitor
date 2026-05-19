# ─── Multi-Stage Rust Build ───
# Benötigt funktionierenden Docker DNS (8.8.8.8) für crates.io-Zugriff
#
# Lokaler Build als Fallback:
#   cargo build --release
#   docker build -t acm-backend -f Dockerfile.local .

FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:latest

COPY --from=builder /app/target/release/Backend /usr/local/bin/Backend
COPY --from=builder /etc/ssl/certs /etc/ssl/certs

EXPOSE 3000
CMD ["/usr/local/bin/Backend"]
