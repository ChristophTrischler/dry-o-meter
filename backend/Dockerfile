# Use an official Rust runtime as a parent image
FROM rust:latest as builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Build the project in release mode
RUN cargo build --release

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
COPY config.toml rinja.toml ./

# Copy the assets and views directories
COPY assets views ./

# Copy the certs directory
COPY certs ./

# Expose the port the app runs on
EXPOSE 8080

# Command to run the application
CMD ["./backend"]