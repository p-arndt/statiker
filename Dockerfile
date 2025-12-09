FROM rust:latest AS builder
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch AS final
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/statiker /app/statiker

# If you need HTTPS/TLS add:
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENTRYPOINT ["/app/statiker"]