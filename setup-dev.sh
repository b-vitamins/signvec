#!/bin/bash
set -euo pipefail

# Setup script for SignVec development environment
# Installs Rust toolchain and fetches dependencies for offline work

main() {
    echo "[setup-dev] Starting setup..."

    if ! command -v curl >/dev/null; then
        echo "[setup-dev] Installing curl"
        apt-get update && apt-get install -y curl
    fi

    echo "[setup-dev] Installing build essentials"
    apt-get update && apt-get install -y build-essential pkg-config git ca-certificates

    if ! command -v rustup >/dev/null; then
        echo "[setup-dev] Installing rustup and Rust toolchain"
        curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        echo "[setup-dev] rustup already installed"
    fi

    source "$HOME/.cargo/env"

    echo "[setup-dev] Ensuring stable toolchain with components"
    rustup toolchain install stable --profile minimal
    rustup component add rustfmt clippy

    echo "[setup-dev] Generating Cargo.lock for reproducibility"
    cargo generate-lockfile

    echo "[setup-dev] Fetching dependencies"
    cargo fetch --locked

    echo "[setup-dev] Building crates to populate cache"
    cargo test --locked --no-run
    cargo bench --bench signvec --no-run || true

    echo "[setup-dev] Setup complete. Rust is located at $(rustc --version)"
}

main "$@"
