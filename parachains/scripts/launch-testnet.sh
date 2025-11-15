#!/bin/bash

# Patient X Parachains - Launch Local Testnet
# This script launches a local testnet with all three parachains using zombienet

set -e

echo "========================================="
echo "Patient X Parachains - Launching Local Testnet"
echo "========================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if zombienet exists
if [ ! -f "./zombienet" ]; then
    echo -e "${RED}Error: zombienet not found. Please run ./scripts/setup.sh first${NC}"
    exit 1
fi

# Check if zombienet config exists
if [ ! -f "./zombienet-config.toml" ]; then
    echo -e "${YELLOW}Creating default zombienet configuration...${NC}"
    cat > zombienet-config.toml << 'EOF'
[relaychain]
default_command = "polkadot"
default_args = ["-lparachain=debug"]
chain = "rococo-local"

  [[relaychain.nodes]]
  name = "alice"
  validator = true
  rpc_port = 9944
  ws_port = 9944

  [[relaychain.nodes]]
  name = "bob"
  validator = true
  rpc_port = 9945
  ws_port = 9945

[[parachains]]
id = 2000
chain = "identity-consent-local"

  [[parachains.collators]]
  name = "identity-consent-collator"
  command = "./identity-consent-chain/target/release/identity-consent-node"
  args = ["-lparachain=debug"]
  rpc_port = 9988
  ws_port = 9988

[[parachains]]
id = 2001
chain = "health-data-local"

  [[parachains.collators]]
  name = "health-data-collator"
  command = "./health-data-chain/target/release/health-data-node"
  args = ["-lparachain=debug"]
  rpc_port = 9989
  ws_port = 9989

[[parachains]]
id = 2002
chain = "marketplace-local"

  [[parachains.collators]]
  name = "marketplace-collator"
  command = "./marketplace-chain/target/release/marketplace-node"
  args = ["-lparachain=debug"]
  rpc_port = 9990
  ws_port = 9990
EOF
    echo -e "${GREEN}✓ Zombienet configuration created${NC}"
fi

# Clean up old data
echo "Cleaning up old chain data..."
rm -rf data/relay/* data/identity-consent/* data/health-data/* data/marketplace/*
echo -e "${GREEN}✓ Old data cleaned${NC}"

echo ""
echo "Starting local testnet with zombienet..."
echo "This will launch:"
echo "  - Relay chain (2 validators: Alice, Bob)"
echo "  - IdentityConsent Chain (Para ID: 2000)"
echo "  - HealthData Chain (Para ID: 2001)"
echo "  - Marketplace Chain (Para ID: 2002)"
echo ""
echo "Endpoints will be:"
echo "  - Relay Chain: ws://localhost:9944"
echo "  - IdentityConsent: ws://localhost:9988"
echo "  - HealthData: ws://localhost:9989"
echo "  - Marketplace: ws://localhost:9990"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the network${NC}"
echo ""

# Launch zombienet
./zombienet spawn --provider native zombienet-config.toml
