#!/bin/bash

# Clean all build artifacts and chain data

set -e

echo "========================================="
echo "Cleaning all build artifacts and data"
echo "========================================="

# Clean build artifacts
echo "Cleaning build artifacts..."
for chain in identity-consent-chain health-data-chain marketplace-chain; do
    if [ -d "$chain" ]; then
        echo "  Cleaning $chain..."
        cd "$chain"
        cargo clean
        cd ..
    fi
done

# Clean chain data
echo "Cleaning chain data..."
rm -rf data/*
rm -rf logs/*

# Clean zombienet artifacts
rm -rf zombie-*
rm -rf /tmp/zombie-*

echo ""
echo "✓ Cleanup complete!"
