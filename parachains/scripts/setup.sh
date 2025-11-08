#!/bin/bash

# Patient X Parachains Setup Script
# This script sets up the development environment for all three parachains

set -e

echo "========================================="
echo "Patient X Parachains - Environment Setup"
echo "========================================="

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "✓ Rust is already installed ($(rustc --version))"
fi

# Add WebAssembly target
echo "Adding WebAssembly target..."
rustup target add wasm32-unknown-unknown
echo "✓ WebAssembly target added"

# Update Rust toolchain
echo "Updating Rust toolchain..."
rustup update stable
rustup update nightly
echo "✓ Rust toolchain updated"

# Check OS and install dependencies
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Detected Linux OS"
    echo "Installing dependencies..."
    sudo apt-get update
    sudo apt-get install -y \
        cmake \
        pkg-config \
        libssl-dev \
        git \
        clang \
        libclang-dev \
        protobuf-compiler \
        build-essential
    echo "✓ Dependencies installed"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Detected macOS"
    if ! command -v brew &> /dev/null; then
        echo "Homebrew is not installed. Please install Homebrew first:"
        echo "/bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
    echo "Installing dependencies..."
    brew install cmake pkg-config openssl git llvm protobuf
    echo "✓ Dependencies installed"
else
    echo "Unsupported OS: $OSTYPE"
    echo "Please manually install: cmake, pkg-config, openssl, git, clang, protobuf-compiler"
    exit 1
fi

# Install Polkadot binary (for local relay chain)
echo "Installing Polkadot binary..."
if ! command -v polkadot &> /dev/null; then
    cargo install --git https://github.com/paritytech/polkadot-sdk.git --tag polkadot-stable2409 polkadot
    echo "✓ Polkadot installed"
else
    echo "✓ Polkadot is already installed ($(polkadot --version))"
fi

# Install zombienet (for local testing)
echo "Installing zombienet..."
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    ZOMBIENET_URL="https://github.com/paritytech/zombienet/releases/latest/download/zombienet-linux-x64"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(uname -m) == "arm64" ]]; then
        ZOMBIENET_URL="https://github.com/paritytech/zombienet/releases/latest/download/zombienet-macos-arm64"
    else
        ZOMBIENET_URL="https://github.com/paritytech/zombienet/releases/latest/download/zombienet-macos-x64"
    fi
fi

if [ ! -f "./zombienet" ]; then
    curl -L -o zombienet "$ZOMBIENET_URL"
    chmod +x zombienet
    echo "✓ Zombienet downloaded"
else
    echo "✓ Zombienet already exists"
fi

# Create necessary directories
echo "Creating directory structure..."
mkdir -p data/relay
mkdir -p data/identity-consent
mkdir -p data/health-data
mkdir -p data/marketplace
mkdir -p logs
echo "✓ Directories created"

# Check if we're in the parachains directory
if [ ! -f "README.md" ]; then
    echo "Error: Please run this script from the parachains directory"
    exit 1
fi

echo ""
echo "========================================="
echo "Setup Complete!"
echo "========================================="
echo ""
echo "Next steps:"
echo "1. Build all parachains: ./scripts/build-all.sh"
echo "2. Launch testnet: ./scripts/launch-testnet.sh"
echo ""
echo "For more information, see README.md"
