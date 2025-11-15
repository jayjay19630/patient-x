#!/bin/bash

# Patient X Parachains - Unified Build, Test, and Run Script
# This script combines setup, build, test, and launch functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
SKIP_SETUP=false
SKIP_BUILD=false
SKIP_TEST=false
SKIP_LAUNCH=false
CHECK_ONLY=false
CLEAN_BUILD=false

# Parse command line arguments
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --skip-setup      Skip environment setup"
    echo "  --skip-build      Skip build process"
    echo "  --skip-test       Skip running tests"
    echo "  --skip-launch     Skip launching testnet"
    echo "  --check-only      Only run cargo check (type checking)"
    echo "  --clean           Clean build (remove target directories first)"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Run everything"
    echo "  $0 --skip-setup --skip-test  # Build and launch only"
    echo "  $0 --check-only              # Only check for type errors"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-setup)
            SKIP_SETUP=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-test)
            SKIP_TEST=true
            shift
            ;;
        --skip-launch)
            SKIP_LAUNCH=true
            shift
            ;;
        --check-only)
            CHECK_ONLY=true
            SKIP_LAUNCH=true
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

echo "========================================="
echo "Patient X Parachains - Unified Runner"
echo "========================================="
echo ""

# Check if we're in the parachains directory
if [ ! -f "README.md" ]; then
    echo -e "${RED}Error: Please run this script from the parachains directory${NC}"
    exit 1
fi

START_TIME=$(date +%s)

# ===========================================
# PHASE 1: SETUP
# ===========================================
if [ "$SKIP_SETUP" = false ]; then
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}PHASE 1: Environment Setup${NC}"
    echo -e "${BLUE}=========================================${NC}"
    ./scripts/setup.sh
else
    echo -e "${YELLOW}Skipping environment setup${NC}"
fi

# ===========================================
# PHASE 2: CLEAN BUILD (if requested)
# ===========================================
if [ "$CLEAN_BUILD" = true ]; then
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}PHASE 2: Cleaning Build Artifacts${NC}"
    echo -e "${BLUE}=========================================${NC}"

    chains=("identity-consent-chain" "health-data-chain" "marketplace-chain")
    for chain in "${chains[@]}"; do
        if [ -d "$chain/target" ]; then
            echo -e "${YELLOW}Cleaning $chain/target...${NC}"
            rm -rf "$chain/target"
            echo -e "${GREEN}✓ $chain cleaned${NC}"
        fi
    done
fi

# ===========================================
# PHASE 3: TYPE CHECKING / BUILD
# ===========================================
if [ "$CHECK_ONLY" = true ]; then
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}PHASE 3: Type Checking All Chains${NC}"
    echo -e "${BLUE}=========================================${NC}"

    check_chain() {
        local chain_name=$1
        local chain_dir=$2

        echo ""
        echo -e "${YELLOW}Checking $chain_name...${NC}"
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

        # Run cargo check on workspace
        echo "Running cargo check on workspace..."
        if cargo check --workspace 2>&1 | tee /tmp/cargo-check-$chain_name.log; then
            echo -e "${GREEN}✓ $chain_name type check passed${NC}"
            cd - > /dev/null
            return 0
        else
            echo -e "${RED}✗ $chain_name has type errors${NC}"
            echo "See /tmp/cargo-check-$chain_name.log for details"
            cd - > /dev/null
            return 1
        fi
    }

    ERRORS=0

    # Check IdentityConsent Chain
    if ! check_chain "IdentityConsent Chain" "identity-consent-chain"; then
        ERRORS=$((ERRORS + 1))
    fi

    # Check HealthData Chain
    if ! check_chain "HealthData Chain" "health-data-chain"; then
        ERRORS=$((ERRORS + 1))
    fi

    # Check Marketplace Chain
    if ! check_chain "Marketplace Chain" "marketplace-chain"; then
        ERRORS=$((ERRORS + 1))
    fi

    echo ""
    echo "========================================="
    if [ $ERRORS -eq 0 ]; then
        echo -e "${GREEN}All chains passed type checking!${NC}"
        echo "========================================="
        exit 0
    else
        echo -e "${RED}$ERRORS chain(s) have type errors${NC}"
        echo "========================================="
        exit 1
    fi

elif [ "$SKIP_BUILD" = false ]; then
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}PHASE 3: Building All Chains${NC}"
    echo -e "${BLUE}=========================================${NC}"
    ./scripts/build-all.sh
else
    echo -e "${YELLOW}Skipping build${NC}"
fi

# ===========================================
# PHASE 4: TESTING
# ===========================================
if [ "$SKIP_TEST" = false ] && [ "$CHECK_ONLY" = false ]; then
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}PHASE 4: Running Tests${NC}"
    echo -e "${BLUE}=========================================${NC}"

    test_chain() {
        local chain_name=$1
        local chain_dir=$2

        echo ""
        echo -e "${YELLOW}Testing $chain_name...${NC}"
        echo "========================================="

        if [ ! -d "$chain_dir" ]; then
            echo -e "${RED}Error: Directory $chain_dir not found${NC}"
            return 1
        fi

        cd "$chain_dir"

        # Run tests
        if cargo test --workspace 2>&1 | tee /tmp/cargo-test-$chain_name.log; then
            echo -e "${GREEN}✓ $chain_name tests passed${NC}"
            cd - > /dev/null
            return 0
        else
            echo -e "${RED}✗ $chain_name tests failed${NC}"
            echo "See /tmp/cargo-test-$chain_name.log for details"
            cd - > /dev/null
            return 1
        fi
    }

    TEST_ERRORS=0

    # Test IdentityConsent Chain
    if ! test_chain "IdentityConsent Chain" "identity-consent-chain"; then
        TEST_ERRORS=$((TEST_ERRORS + 1))
    fi

    # Test HealthData Chain
    if ! test_chain "HealthData Chain" "health-data-chain"; then
        TEST_ERRORS=$((TEST_ERRORS + 1))
    fi

    # Test Marketplace Chain
    if ! test_chain "Marketplace Chain" "marketplace-chain"; then
        TEST_ERRORS=$((TEST_ERRORS + 1))
    fi

    echo ""
    echo "========================================="
    if [ $TEST_ERRORS -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
    else
        echo -e "${YELLOW}Warning: $TEST_ERRORS chain(s) had test failures${NC}"
        echo -e "${YELLOW}Continuing to launch phase...${NC}"
    fi
    echo "========================================="
else
    echo -e "${YELLOW}Skipping tests${NC}"
fi

# ===========================================
# PHASE 5: LAUNCH TESTNET
# ===========================================
if [ "$SKIP_LAUNCH" = false ]; then
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}PHASE 5: Launching Testnet${NC}"
    echo -e "${BLUE}=========================================${NC}"
    ./scripts/launch-testnet.sh
else
    echo -e "${YELLOW}Skipping testnet launch${NC}"
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "========================================="
echo -e "${GREEN}All Operations Complete!${NC}"
echo "========================================="
echo "Total time: ${DURATION} seconds"
echo ""
