# Enable buildkit features
# syntax = docker/dockerfile:1.4

########################################
# 1. Base image with necessary tools
########################################
FROM rust:slim-bookworm AS base

# Install system dependencies and clean up
RUN apt-get update \
    && apt-get install -y \
       pkg-config \
       libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-chef and sccache for dependency caching
RUN cargo install cargo-chef --version ^0.1 \
    && cargo install sccache

########################################
# 2. Planner stage (cargo-chef)
########################################
FROM base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

########################################
# 3. Builder stage
########################################
FROM base AS builder
WORKDIR /app

# Copy the prepared recipe and cook dependencies
COPY --from=planner /app/recipe.json ./recipe.json

# Configure sccache
ENV SCCACHE_DIR=/sccache
ENV SCCACHE_CACHE_SIZE=5G

# Cook dependencies using cargo-chef with caching
RUN --mount=type=cache,target=/sccache \
    cargo chef cook --release --recipe-path recipe.json

# Install additional build-time dependencies
RUN apt-get update \
    && apt-get install -y \
       protobuf-compiler \
       libpq-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Build the application
COPY . .
RUN --mount=type=cache,target=/sccache \
    cargo build --release

# Output sccache statistics
RUN sccache --show-stats

########################################
# 4. Runtime stage
########################################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install only runtime dependencies and clean up
RUN apt-get update \
    && apt-get install -y \
       libpq-dev \
       ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -ms /bin/bash rustuser
RUN chown -R rustuser:rustuser /app
USER rustuser

# Copy built binary and config
RUN mkdir -p bin config
COPY --from=builder /app/target/release/grpc-server bin/grpc-server
COPY --from=builder /app/config config

ENTRYPOINT ["/app/bin/grpc-server"]
