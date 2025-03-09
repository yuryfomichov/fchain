FROM mcr.microsoft.com/devcontainers/rust:latest

# Install additional packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for development
RUN cargo install cargo-watch

# Set up a non-root user
USER vscode

# Set the working directory
WORKDIR /workspace 