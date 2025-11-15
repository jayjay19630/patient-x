#!/bin/bash

# Patient X Parachains - One Command Setup and Run
# This is the EASIEST way to get started

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║        Patient X Parachains - Quick Start         ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Step 1: Check prerequisites
echo -e "${YELLOW}[1/4] Checking prerequisites...${NC}"

if ! command_exists cargo; then
    echo -e "${RED}✗ Rust not found${NC}"
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

echo -e "${GREEN}✓ Rust installed${NC}"

# Add wasm target
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Step 2: Install dependencies
echo -e "${YELLOW}[2/4] Installing system dependencies...${NC}"

if [[ "$OSTYPE" == "darwin"* ]]; then
    if command_exists brew; then
        brew install cmake pkg-config openssl protobuf 2>/dev/null || true
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if command_exists apt-get; then
        sudo apt-get update -qq
        sudo apt-get install -y -qq cmake pkg-config libssl-dev git clang libclang-dev protobuf-compiler build-essential
    fi
fi

echo -e "${GREEN}✓ Dependencies ready${NC}"

# Step 3: Build (if needed)
echo -e "${YELLOW}[3/4] Building parachains...${NC}"

if [ ! -f "identity-consent-chain/target/release/identity-consent-node" ]; then
    echo "First time setup - this will take 20-40 minutes..."
    echo "Building all chains..."

    chmod +x scripts/build-all.sh
    ./scripts/build-all.sh || {
        echo -e "${YELLOW}Note: Some chains may not have full node implementations yet${NC}"
        echo -e "${YELLOW}Continuing with available chains...${NC}"
    }
else
    echo -e "${GREEN}✓ Binaries already built${NC}"
fi

# Step 4: Install Polkadot binary
echo -e "${YELLOW}[4/5] Installing Polkadot...${NC}"

if ! command_exists polkadot; then
    echo "Polkadot not found. Installing (this takes 15-30 minutes)..."
    cargo install --git https://github.com/paritytech/polkadot-sdk.git --tag polkadot-stable2409 polkadot || {
        echo -e "${RED}Warning: Polkadot installation failed${NC}"
        echo "You can install it manually later with:"
        echo "  cargo install --git https://github.com/paritytech/polkadot-sdk.git --tag polkadot-stable2409 polkadot"
    }
else
    echo -e "${GREEN}✓ Polkadot already installed ($(polkadot --version | head -1))${NC}"
fi

# Step 5: Download zombienet if needed
echo -e "${YELLOW}[5/5] Setting up zombienet...${NC}"

if [ ! -f "./zombienet" ]; then
    echo "Downloading zombienet..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if [[ $(uname -m) == "arm64" ]]; then
            URL="https://github.com/paritytech/zombienet/releases/latest/download/zombienet-macos-arm64"
        else
            URL="https://github.com/paritytech/zombienet/releases/latest/download/zombienet-macos-x64"
        fi
    else
        URL="https://github.com/paritytech/zombienet/releases/latest/download/zombienet-linux-x64"
    fi

    curl -L -o zombienet "$URL"
    chmod +x zombienet
fi

echo -e "${GREEN}✓ Zombienet ready${NC}"

# Create data directories
mkdir -p data/{relay,identity-consent,health-data,marketplace} logs

# Launch!
echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║              Launching Testnet...                 ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""
echo "This will start:"
echo "  • Relay Chain (2 validators)"
echo "  • IdentityConsent Chain (Para 2000) - ws://localhost:9988"
echo "  • HealthData Chain (Para 2001) - ws://localhost:9989"
echo "  • Marketplace Chain (Para 2002) - ws://localhost:9990"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all chains${NC}"
echo ""
echo "Connect with Polkadot.js Apps:"
echo "  https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9988"
echo ""

sleep 2

# Launch zombienet
if [ ! -f "zombienet-config.toml" ]; then
    echo "Creating zombienet configuration..."
    cat > zombienet-config.toml << 'EOF'
[relaychain]
default_command = "polkadot"
default_args = ["-lparachain=debug"]
chain = "rococo-local"

  [[relaychain.nodes]]
  name = "alice"
  validator = true
  rpc_port = 9944

  [[relaychain.nodes]]
  name = "bob"
  validator = true
  rpc_port = 9945

[[parachains]]
id = 2000
chain = "identity-consent-local"

  [[parachains.collators]]
  name = "identity-consent-collator"
  command = "./identity-consent-chain/target/release/identity-consent-node"
  args = ["-lparachain=debug"]
  rpc_port = 9988

[[parachains]]
id = 2001
chain = "health-data-local"

  [[parachains.collators]]
  name = "health-data-collator"
  command = "./health-data-chain/target/release/health-data-node"
  args = ["-lparachain=debug"]
  rpc_port = 9989

[[parachains]]
id = 2002
chain = "marketplace-local"

  [[parachains.collators]]
  name = "marketplace-collator"
  command = "./marketplace-chain/target/release/marketplace-node"
  args = ["-lparachain=debug"]
  rpc_port = 9990
EOF
fi

./zombienet spawn --provider native zombienet-config.toml
