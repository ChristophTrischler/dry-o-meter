# Use an official Rust runtime as a parent image
FROM rust:latest AS builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Create a dummy project and build dependencies to cache them
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy the actual source code, migrations, and offline metadata
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY assets ./assets

# Enforce offline build mode
ENV SQLX_OFFLINE=true

# Build the project in release mode (touch main.rs to force cargo to rebuild)
RUN touch src/main.rs && cargo build --release

# Use a lightweight image for the final stage
FROM debian:stable-slim

# Install required dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/backend .

# Copy the configuration files
COPY config.docker.toml ./

# Copy assets directory
COPY assets ./assets

# Expose the port the app runs on
EXPOSE 8008

# Command to run the application
CMD ["./backend"]