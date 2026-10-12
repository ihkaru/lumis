# ==============================================================================
# Lumis Enterprise Multi-Stage Container Image (<35MB Runtime Footprint)
# Zero-Vendor Lock-in: Runs on Docker, Podman, Kubernetes, and Coolify
# ==============================================================================

# Stage 1: Build Svelte 5 Frontend via Bun
FROM oven/bun:1.2-alpine AS frontend-builder
WORKDIR /app/ui

# Install dependencies and build static distribution
COPY ui/package.json ./
RUN bun install
COPY ui/ ./
RUN bun run build

# Stage 2: Build Pure-Rust Backend Binaries
FROM rust:1.85-slim-bookworm AS backend-builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev cmake build-essential \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml ./
COPY Cargo.lock* ./
COPY crates/lumis-core/Cargo.toml crates/lumis-core/
COPY crates/lumis-db/Cargo.toml crates/lumis-db/
COPY crates/lumis-server/Cargo.toml crates/lumis-server/

# Copy full source and build optimized release binary
COPY crates/ ./crates/
RUN cargo build --release --bin lumis-server

# Stage 3: Minimal Distroless / Hardened Slim Runtime
FROM debian:bookworm-slim AS runtime

# Install CA certificates for TLS to external LLM providers
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged application user
RUN groupadd -g 10001 lumis && \
    useradd -u 10001 -g lumis -s /bin/false -m lumis

WORKDIR /app

# Copy compiled backend binary
COPY --from=backend-builder /app/target/release/lumis-server /usr/local/bin/lumis-server

# Copy built frontend assets
COPY --from=frontend-builder /app/ui/dist /app/ui/dist

# Create persistent storage directory for SQLite/data
RUN mkdir -p /data && chown -R lumis:lumis /data /app

# Switch to unprivileged security context
USER 10001:10001

# Environment Defaults (12-Factor Compliant)
ENV PORT=3000 \
    DATABASE_URL="sqlite:///data/lumis.sqlite?mode=rwc" \
    STATIC_DIR="/app/ui/dist" \
    TRUSTED_HEADER="x-forwarded-user" \
    RUST_LOG="lumis_server=info,tower_http=info"

EXPOSE 3000

# Container Healthcheck (Liveness Probe)
HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["/usr/local/bin/lumis-server"]
