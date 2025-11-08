#!/bin/bash

# Patient X Parachains - Build All Chains
# This script builds all three parachains

set -e

echo "========================================="
echo "Patient X Parachains - Building All Chains"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to build a parachain
build_parachain() {
    local chain_name=$1
    local chain_dir=$2

    echo ""
    echo -e "${YELLOW}Building $chain_name...${NC}"
    echo "========================================="

    if [ ! -d "$chain_dir" ]; then
        echo -e "${RED}Error: Directory $chain_dir not found${NC}"
        return 1
    fi

    cd "$chain_dir"

    # Check if Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}Error: Cargo.toml not found in $chain_dir${NC}"
        cd - > /dev/null
        return 1
    fi

    # Build runtime first
    if [ -d "runtime" ]; then
        echo "Building runtime..."
        cd runtime
        cargo build --release
        cd ..
    fi

    # Build the full node (if node directory exists)
    if [ -d "node" ]; then
        echo "Building node..."
        cd node
        cargo build --release
        cd ..
    else
        # Build from root if no node directory
        echo "Building from root..."
        cargo build --release
    fi

    cd - > /dev/null

    echo -e "${GREEN}✓ $chain_name built successfully${NC}"
    return 0
}

# Check if we're in the parachains directory
if [ ! -f "README.md" ]; then
    echo -e "${RED}Error: Please run this script from the parachains directory${NC}"
    exit 1
fi

START_TIME=$(date +%s)

# Build IdentityConsent Chain
if ! build_parachain "IdentityConsent Chain" "identity-consent-chain"; then
    echo -e "${YELLOW}Note: IdentityConsent Chain build skipped or failed${NC}"
    echo -e "${YELLOW}This is expected if you haven't generated the node implementation yet${NC}"
fi

# Build HealthData Chain
if ! build_parachain "HealthData Chain" "health-data-chain"; then
    echo -e "${YELLOW}Note: HealthData Chain build skipped or failed${NC}"
    echo -e "${YELLOW}This is expected if you haven't generated the node implementation yet${NC}"
fi

# Build Marketplace Chain
if ! build_parachain "Marketplace Chain" "marketplace-chain"; then
    echo -e "${YELLOW}Note: Marketplace Chain build skipped or failed${NC}"
    echo -e "${YELLOW}This is expected if you haven't generated the node implementation yet${NC}"
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "========================================="
echo -e "${GREEN}Build Process Complete!${NC}"
echo "========================================="
echo "Total time: ${DURATION} seconds"
echo ""
echo "Note: The node binaries should be located at:"
echo "  - identity-consent-chain/target/release/identity-consent-node"
echo "  - health-data-chain/target/release/health-data-node"
echo "  - marketplace-chain/target/release/marketplace-node"
echo ""
echo "Next step: ./scripts/launch-testnet.sh"
