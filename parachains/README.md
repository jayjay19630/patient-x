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
├── docker-compose.yml                 # Local testnet setup
├── scripts/
│   ├── setup.sh                      # Environment setup
│   ├── build-all.sh                  # Build all parachains
│   ├── launch-testnet.sh             # Launch local testnet
│   └── register-parachains.sh        # Register parachains
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
brew install cmake pkg-config openssl git llvm

# On Ubuntu/Debian
sudo apt install -y cmake pkg-config libssl-dev git clang libclang-dev protobuf-compiler
```

### Build All Chains

```bash
cd parachains
chmod +x scripts/*.sh
./scripts/build-all.sh
```

### Launch Local Testnet

```bash
# Start local relay chain and all three parachains
./scripts/launch-testnet.sh
```

### Access Endpoints

- **Relay Chain**: ws://localhost:9944
- **IdentityConsent Chain**: ws://localhost:9988
- **HealthData Chain**: ws://localhost:9989
- **Marketplace Chain**: ws://localhost:9990

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

## Deployment

### Testnet Deployment (Rococo)

1. Build parachain artifacts
2. Generate chain spec
3. Register parachain on Rococo
4. Start collators

See [docs/deployment.md](./docs/deployment.md) for detailed instructions.

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
