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
│   ├── run.sh                        # One-command setup & launch
│   ├── build-all.sh                  # Build all parachains
│   └── clean-all.sh                  # Clean build artifacts
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

## Quick Start

### One-Command Setup (Easiest Way)

```bash
cd parachains
chmod +x scripts/run.sh
./scripts/run.sh
```

This single command will:
1. Install Rust and all dependencies
2. Build all three parachains (~30-40 min first time)
3. Launch local testnet with relay chain and parachains

### Manual Setup

If you prefer step-by-step control:

#### 1. Install Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install system dependencies
# macOS:
brew install cmake pkg-config openssl git llvm protobuf

# Ubuntu/Debian:
sudo apt install -y cmake pkg-config libssl-dev git clang libclang-dev protobuf-compiler build-essential
```

#### 2. Build All Chains

```bash
cd parachains
chmod +x scripts/build-all.sh
./scripts/build-all.sh
```

Expected build time: 20-40 minutes (first build)

#### 3. Launch Testnet

The build script will output instructions, but you can manually launch with:

```bash
# Download and run polkadot relay chain + zombienet
# (Or just use run.sh which does everything)
```

### Access the Chains

Once running, connect with [Polkadot.js Apps](https://polkadot.js.org/apps/):

- **Relay Chain**: [ws://localhost:9944](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944)
- **IdentityConsent Chain**: [ws://localhost:9988](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9988)
- **HealthData Chain**: [ws://localhost:9989](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9989)
- **Marketplace Chain**: [ws://localhost:9990](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9990)

## Usage Examples

### 1. Register Identity (IdentityConsent Chain)

Connect to ws://localhost:9988 in Polkadot.js Apps:

```javascript
// Developer → Extrinsics
identityRegistry.registerIdentity(
  did: "did:patientx:alice:123456",
  role: "Patient",
  name: "Alice Patient",
  emailHash: "0x..." // keccak256 hash of email
)
```

### 2. Create Consent

```javascript
consentManager.createConsent(
  consumer: "5GrwvaEF...", // Researcher's account
  purpose: "Research",
  dataTypes: ["LabResults", "Genomic"],
  expiresAt: 1735689600, // Unix timestamp
  termsHash: "0x..." // Hash of consent terms
)
```

### 3. Store Health Record (HealthData Chain)

Connect to ws://localhost:9989:

```javascript
healthRecords.storeRecord(
  ipfsHash: "QmX...", // IPFS CID
  encryptionKeyHash: "0x...",
  recordType: "LabResults",
  metadata: "{...}"
)
```

### 4. List Data (Marketplace Chain)

Connect to ws://localhost:9990:

```javascript
dataListings.createListing(
  dataHash: "0x...",
  price: 1000000000000,
  consentId: "0x...",
  description: "Lab results dataset"
)
```

## Architecture Diagram

```
┌──────────────────────────────────────────────────┐
│         Polkadot Relay Chain (Local)             │
│         Alice (9944) & Bob (9945)                │
└────────┬──────────────┬─────────────┬────────────┘
         │              │             │
    ┌────▼────┐    ┌────▼────┐   ┌───▼─────┐
    │Identity │    │ Health  │   │Marketplace│
    │Consent  │◄──►│  Data   │◄─►│  Chain   │
    │(2000)   │    │ (2001)  │   │  (2002)  │
    │:9988    │    │ :9989   │   │  :9990   │
    └─────────┘    └─────────┘   └──────────┘
         XCM Messages Flow Between All Chains
```

## Development

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
./scripts/test-all.sh

# Test individual chain
cd identity-consent-chain
cargo test
```

### Clean Build Artifacts

```bash
./scripts/clean-all.sh
```

## Troubleshooting

### Build Issues

**Problem**: Build fails with linker errors
```bash
# macOS
brew install llvm

# Linux
sudo apt install build-essential clang
```

**Problem**: "WASM binary not available"
```bash
cargo clean
cargo build --release
```

### Runtime Issues

**Problem**: Ports already in use
```bash
# Kill processes on ports
lsof -ti:9944,9988,9989,9990 | xargs kill -9
```

**Problem**: Parachain won't connect
- Ensure relay chain is producing blocks first
- Check that para IDs match in config
- Verify collator is running

### Development Tips

1. **Fast iterations**: Use `cargo check` instead of `cargo build` for syntax checking
2. **Watch mode**: Use `cargo watch -x check` for auto-compilation
3. **Parallel builds**: Use `cargo build -j4` to specify job count
4. **Clean state**: Delete `data/` directory between runs for fresh state

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

## Future Enhancements

1. Zero-knowledge proofs for privacy-preserving analytics
2. Integration with Polkadot's identity pallet
3. Machine learning model marketplace
4. Cross-chain bridges to Ethereum/Cosmos
5. Mobile SDK for patient apps
6. Federated learning support

## License

Apache 2.0

## Support

For issues and questions, please open a GitHub issue or contact the development team.
