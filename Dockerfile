FROM rust:1.77-slim as builder

WORKDIR /usr/src/webcheck
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/webcheck

COPY --from=builder /usr/src/webcheck/target/release/webcheck /opt/webcheck/webcheck
COPY --from=builder /usr/src/webcheck/templates /opt/webcheck/templates

# Create data directory for configuration
RUN mkdir -p /opt/webcheck/data && \
    chmod 755 /opt/webcheck/data

# Set working directory to data directory so config files are saved there
WORKDIR /opt/webcheck/data

EXPOSE 3000

CMD ["/opt/webcheck/webcheck"]