# syntax=docker/dockerfile:1.7

# Use an official Rust runtime as a parent image
FROM rust:latest AS builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy manifests, source code, migrations, and offline metadata
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY assets ./assets

# Enforce offline build mode
ENV SQLX_OFFLINE=true

# Build in release mode with persistent cache mounts, then copy the binary out
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release && cp target/release/backend /usr/src/app/backend-bin

# Use a lightweight image for the final stage
FROM debian:stable-slim

# Install required dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/backend-bin ./backend

# Copy the configuration files
COPY config.docker.toml ./

# Copy assets directory
COPY assets ./assets

# Expose the port the app runs on
EXPOSE 8008

# Command to run the application
CMD ["./backend"]
