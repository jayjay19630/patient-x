# Patient X - Medical Data Marketplace Parachains

## Overview

Patient X is a decentralized medical data marketplace built on three interconnected Polkadot parachains:

1. **IdentityConsent Chain** - Identity, authentication, and consent management
2. **HealthData Chain** - Medical record storage, encryption, and access control
3. **Marketplace Chain** - Data discovery, listing, and economic transactions

## Architecture

### 1. IdentityConsent Chain (Para ID: 2000)

**Purpose**: Manages user identity, authentication, and consent smart contracts

**Core Pallets**:
- `pallet-identity-registry` - User identity management (patients, researchers, institutions)
- `pallet-consent-manager` - Consent smart contracts and policies
- `pallet-authentication` - Authentication and authorization
- Standard pallets: `frame-system`, `pallet-balances`, `pallet-timestamp`, `pallet-xcm`

**Key Features**:
- Self-sovereign identity (DID-based)
- Granular consent management (purpose, duration, data types)
- Consent revocation and expiry
- Role-based access control (Patient, Researcher, Institution, Auditor)

### 2. HealthData Chain (Para ID: 2001)

**Purpose**: Anchors/encrypts medical records, integrates with IPFS, enforces access based on consent

**Core Pallets**:
- `pallet-health-records` - Medical record anchoring and metadata
- `pallet-ipfs-integration` - IPFS content addressing and pinning
- `pallet-access-control` - Access enforcement based on consent
- `pallet-encryption` - Encryption key management
- Standard pallets: `frame-system`, `pallet-balances`, `pallet-timestamp`, `pallet-xcm`

**Key Features**:
- IPFS integration for decentralized storage
- Encrypted data storage with key management
- Consent-based access control (queries IdentityConsent Chain via XCM)
- Audit trail for all data access
- Support for multiple data formats (FHIR, DICOM, HL7)

### 3. Marketplace Chain (Para ID: 2002)

**Purpose**: Data discovery, listing, economic activity, and cross-chain transactions

**Core Pallets**:
- `pallet-data-listings` - Data set listings and discovery
- `pallet-marketplace` - Transactions and payments
- `pallet-reputation` - User reputation and ratings
- `pallet-analytics` - Usage analytics and metrics
- Standard pallets: `frame-system`, `pallet-balances`, `pallet-timestamp`, `pallet-xcm`

**Key Features**:
- Data discovery and search
- Dynamic pricing mechanisms
- Payment processing with escrow
- Reputation system for data quality
- Analytics for market insights

## Cross-Chain Communication (XCM)

### Message Flows

1. **Data Access Request**:
   ```
   Marketplace → IdentityConsent: Check consent status
   IdentityConsent → Marketplace: Return consent approval
   Marketplace → HealthData: Request data access
   HealthData → IdentityConsent: Verify consent (double-check)
   HealthData → Marketplace: Return encrypted data pointer
   ```

2. **Consent Update**:
   ```
   Patient via IdentityConsent: Update/revoke consent
   IdentityConsent → HealthData: Notify consent change
   IdentityConsent → Marketplace: Update listing availability
   ```

3. **Payment Flow**:
   ```
   Researcher via Marketplace: Purchase data access
   Marketplace → IdentityConsent: Verify consent
   Marketplace: Process payment (escrow)
   Marketplace → HealthData: Grant temporary access
   HealthData: Log access in audit trail
   Marketplace: Release payment to data owner
   ```

## Technology Stack

- **Framework**: Polkadot SDK (Substrate)
- **Parachain**: Cumulus
- **Consensus**: Aura (for collators) + GRANDPA (finality from relay chain)
- **Storage**: IPFS for off-chain data
- **Encryption**: ChaCha20-Poly1305 for data, X25519 for key exchange
- **Messaging**: XCM v3
- **Smart Contracts**: ink! (optional for advanced consent logic)

## Directory Structure

```
parachains/
├── README.md                          # This file
├── scripts/
│   ├── run-all.sh                    # Unified script (setup, build, test, launch)
│   ├── setup.sh                      # Environment setup
│   ├── build-all.sh                  # Build all parachains
│   └── launch-testnet.sh             # Launch local testnet
├── data/                              # Chain data (created by setup.sh)
│   ├── relay/
│   ├── identity-consent/
│   ├── health-data/
│   └── marketplace/
├── logs/                              # Runtime logs
├── identity-consent-chain/
│   ├── Cargo.toml
│   ├── node/                         # Node implementation
│   ├── runtime/                      # Runtime configuration
│   └── pallets/
│       ├── identity-registry/
│       ├── consent-manager/
│       └── authentication/
├── health-data-chain/
│   ├── Cargo.toml
│   ├── node/
│   ├── runtime/
│   └── pallets/
│       ├── health-records/
│       ├── ipfs-integration/
│       ├── access-control/
│       └── encryption/
└── marketplace-chain/
    ├── Cargo.toml
    ├── node/
    ├── runtime/
    └── pallets/
        ├── data-listings/
        ├── marketplace/
        ├── reputation/
        └── analytics/
```

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Polkadot SDK dependencies
# On macOS
brew install cmake pkg-config openssl git llvm protobuf

# On Ubuntu/Debian
sudo apt install -y cmake pkg-config libssl-dev git clang libclang-dev protobuf-compiler
```

### Quick Start (All-in-One)

The easiest way to get started is using the unified `run-all.sh` script:

```bash
cd parachains
chmod +x scripts/*.sh

# Run everything: setup, build, test, and launch
./scripts/run-all.sh

# Or skip certain phases
./scripts/run-all.sh --skip-setup --skip-test

# Just check for type errors
./scripts/run-all.sh --check-only

# Build without tests
./scripts/run-all.sh --skip-test --skip-launch

# Clean build
./scripts/run-all.sh --clean
```

**Available Options:**
- `--skip-setup` - Skip environment setup (Rust, dependencies, tools)
- `--skip-build` - Skip building the chains
- `--skip-test` - Skip running tests
- `--skip-launch` - Skip launching the testnet
- `--check-only` - Only run type checking (cargo check)
- `--clean` - Remove target directories before building
- `-h, --help` - Show help message

### Manual Setup

If you prefer to run each step manually:

#### 1. Environment Setup

```bash
cd parachains
./scripts/setup.sh
```

This installs:
- Rust toolchain and WebAssembly target
- System dependencies (cmake, pkg-config, openssl, etc.)
- Polkadot binary (for local relay chain)
- Zombienet (for testnet orchestration)
- Creates necessary directories

#### 2. Build All Chains

```bash
./scripts/build-all.sh
```

#### 3. Run Tests

```bash
# Test individual chain
cd identity-consent-chain
cargo test --workspace

# Or test all via run-all.sh
./scripts/run-all.sh --skip-setup --skip-build --skip-launch
```

#### 4. Launch Local Testnet

```bash
./scripts/launch-testnet.sh
```

### Access Endpoints

Once the testnet is running:

- **Relay Chain**: ws://localhost:9944
- **IdentityConsent Chain**: ws://localhost:9988
- **HealthData Chain**: ws://localhost:9989
- **Marketplace Chain**: ws://localhost:9990

## Development

### Type Checking

```bash
# Check all chains for type errors
./scripts/run-all.sh --check-only --skip-setup

# Check individual chain
cd identity-consent-chain
cargo check --workspace
```

### Building Individual Chains

```bash
# IdentityConsent Chain
cd identity-consent-chain
cargo build --release

# HealthData Chain
cd health-data-chain
cargo build --release

# Marketplace Chain
cd marketplace-chain
cargo build --release
```

### Running Tests

```bash
# Test all chains
for chain in identity-consent-chain health-data-chain marketplace-chain; do
  cd $chain
  cargo test --workspace
  cd ..
done

# Test specific pallet
cd identity-consent-chain/pallets/identity-registry
cargo test
```

## Security Considerations

1. **Data Encryption**: All medical data is encrypted before IPFS storage
2. **Key Management**: Encryption keys are managed on-chain with secure access
3. **Consent Verification**: Double-verification via XCM for critical operations
4. **Audit Trail**: All data access is logged immutably
5. **Privacy**: Zero-knowledge proofs for sensitive queries (future enhancement)

## Compliance

- **HIPAA**: End-to-end encryption, access controls, audit logs
- **GDPR**: Right to erasure (pointer removal), consent management
- **HITECH**: Breach notification through on-chain events

## Scripts

The `scripts/` directory contains:

- **[run-all.sh](scripts/run-all.sh)** - Unified script to run everything (setup, build, test, launch)
- **[setup.sh](scripts/setup.sh)** - Install development environment and dependencies
- **[build-all.sh](scripts/build-all.sh)** - Build all three parachains
- **[launch-testnet.sh](scripts/launch-testnet.sh)** - Launch local testnet with zombienet
