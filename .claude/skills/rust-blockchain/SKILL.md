---
name: rust-blockchain
description: Comprehensive guide for building sophisticated blockchain applications using Rust. Covers Substrate framework, Solana development with Anchor, smart contract patterns, security best practices, testing strategies, and production-grade system architecture. Use when building blockchain infrastructure, smart contracts, pallets, Solana programs, implementing consensus mechanisms, designing token economies, or architecting decentralized applications. Includes setup, development workflows, security patterns, deployment strategies, and optimization techniques for senior-level blockchain engineering.
license: MIT
---

# Rust Blockchain Development

Comprehensive skill for building production-grade blockchain applications with Rust, covering multiple frameworks, security patterns, and architectural best practices.

## Quick Start

### Project Type Selection

Determine which blockchain platform to use:

**Substrate (Polkadot Ecosystem)**
- Building custom blockchains or parachains
- Need forkless runtime upgrades
- Require modular, composable architecture
- Enterprise-grade blockchain solutions
- See: [references/substrate-development.md](references/substrate-development.md)

**Solana (High-Performance)**
- Need 50,000+ TPS throughput
- Building DeFi protocols or NFT platforms
- Require sub-second finality
- Cost-sensitive applications ($0.00025/tx)
- See: [references/solana-development.md](references/solana-development.md)

**NEAR Protocol**
- WebAssembly-based smart contracts
- Focus on developer experience
- Sharded blockchain architecture

**CosmWasm (Cosmos Ecosystem)**
- Inter-blockchain communication (IBC)
- Multi-chain smart contracts
- CosmWasm-compatible contracts

## Development Workflow

### 1. Environment Setup

Before starting any blockchain project, set up the development environment:

```bash
# Install Rust and tooling
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Platform-specific setup
# For detailed instructions, see:
```

**→ Read [references/setup-environment.md](references/setup-environment.md)** for complete setup instructions including:
- Rust configuration and toolchains
- Substrate dependencies and node setup
- Solana CLI and Anchor installation
- IDE configuration and debugging
- Security setup and key management

### 2. Project Initialization

**Substrate Project:**
```bash
# Clone node template
git clone https://github.com/substrate-developer-hub/substrate-node-template
cd substrate-node-template

# Build and run
cargo build --release
./target/release/node-template --dev
```

**Solana/Anchor Project:**
```bash
# Create new Anchor project
anchor init my-project
cd my-project

# Start local validator
solana-test-validator

# Build and deploy
anchor build
anchor deploy
anchor test
```

### 3. Development Process

Follow this iterative development cycle:

#### A. Design Phase
- Define state structures and storage layout
- Design instruction/extrinsic interfaces
- Plan error handling and events
- Consider security implications upfront

#### B. Implementation Phase
- Write core business logic
- Implement state transitions
- Add comprehensive error handling
- Follow security best practices from the start

**→ Read [references/security-best-practices.md](references/security-best-practices.md)** for:
- Common vulnerabilities and how to prevent them
- Input validation patterns
- Access control implementation
- Safe arithmetic operations
- Reentrancy protection

**→ Read [references/smart-contract-patterns.md](references/smart-contract-patterns.md)** for:
- Structural patterns (Factory, Registry, Proxy)
- Behavioral patterns (State Machine, Oracle, Event Sourcing)
- Security patterns (Pull Payment, Circuit Breaker, Rate Limiting)
- Economic patterns (Vesting, Staking)

#### C. Testing Phase
- Write unit tests for all functions
- Create integration tests for workflows
- Perform property-based testing
- Run security audits

**→ Read [references/testing-deployment.md](references/testing-deployment.md)** for:
- Comprehensive testing strategies
- Unit and integration test examples
- Property-based testing with proptest
- Fuzzing techniques
- Load testing approaches

#### D. Optimization Phase
- Profile and optimize hot paths
- Minimize storage usage
- Optimize transaction weights/fees
- Review gas/compute costs

### 4. Security Review

**CRITICAL:** Before deploying to testnet or mainnet, conduct thorough security review:

```rust
// Security Checklist:
// ✅ All inputs validated
// ✅ Checked arithmetic everywhere
// ✅ No reentrancy vulnerabilities
// ✅ Access control properly enforced
// ✅ No unbounded loops or storage
// ✅ Events emitted for critical operations
// ✅ Error handling comprehensive
// ✅ Tests cover edge cases and attack vectors
```

Run automated security tools:
```bash
cargo audit              # Check for vulnerable dependencies
cargo clippy -- -D warnings  # Strict linting
cargo test --all-features    # All tests pass
```

### 5. Deployment

Deploy following this progression:

**Development → Testnet → Mainnet**

1. **Development network**: Test all functionality locally
2. **Testnet**: Deploy to public testnet, monitor for 1-2 weeks
3. **Mainnet**: Gradual rollout with monitoring

**→ See [references/testing-deployment.md](references/testing-deployment.md)** for detailed deployment strategies.

## Core Capabilities

### 1. Substrate Pallet Development

Build modular blockchain components (pallets) for Substrate-based chains.

**Key features:**
- Custom storage structures with bounded collections
- Dispatchable functions (extrinsics)
- Events and errors
- Runtime configuration
- Hooks (on_initialize, on_finalize, offchain_worker)
- Benchmarking for transaction weights

**Example use cases:**
- "Create a pallet for managing user profiles with reputation scores"
- "Build a DAO governance pallet with proposal voting"
- "Implement a custom token standard with vesting"

**→ Read [references/substrate-development.md](references/substrate-development.md)** for comprehensive guide including:
- Complete pallet structure and examples
- Storage patterns and best practices
- Runtime configuration
- Testing with mock runtime
- Benchmarking for weights

### 2. Solana Program Development

Build high-performance programs for Solana using native Rust or Anchor framework.

**Key features:**
- Account-based architecture
- Program Derived Addresses (PDAs)
- Cross-Program Invocation (CPI)
- Anchor framework abstractions
- SPL token integration

**Example use cases:**
- "Create an NFT marketplace with bidding functionality"
- "Build a staking program with reward distribution"
- "Implement a decentralized exchange with liquidity pools"

**→ Read [references/solana-development.md](references/solana-development.md)** for:
- Anchor framework patterns
- Account validation strategies
- PDA derivation and usage
- CPI implementation
- Testing with Anchor's test suite

### 3. Smart Contract Security

Apply industry-standard security practices to prevent vulnerabilities.

**Critical security patterns:**
- Input validation and sanitization
- Checked arithmetic to prevent overflow
- Access control and authorization
- Reentrancy protection (checks-effects-interactions)
- Rate limiting and circuit breakers
- Pull payment over push

**→ Read [references/security-best-practices.md](references/security-best-practices.md)** for:
- Common vulnerability patterns and fixes
- Comprehensive input validation
- Role-based access control (RBAC)
- Time-locked operations
- Cryptographic best practices
- Complete security audit checklist

### 4. Testing & Quality Assurance

Implement comprehensive testing strategies for blockchain applications.

**Testing levels:**
- **Unit tests**: Function-level correctness
- **Integration tests**: Cross-module interactions
- **Property-based tests**: Invariant checking
- **Fuzzing**: Random input testing
- **Load tests**: Performance under stress
- **Security tests**: Vulnerability scanning

**→ Read [references/testing-deployment.md](references/testing-deployment.md)** for complete testing guide.

### 5. System Architecture

Design scalable, maintainable blockchain systems.

**Architecture patterns:**
- Layered architecture (Application, Business Logic, Data)
- Event-driven microservices
- State management and versioning
- P2P networking with libp2p
- Sharding and Layer 2 solutions
- Cross-chain bridges

**→ Read [references/system-architecture.md](references/system-architecture.md)** for:
- Production-grade system design
- Scalability patterns
- Network architecture
- High availability setup
- Monitoring and maintenance

## Common Development Tasks

### Creating a Custom Token

**Substrate:**
```rust
// Implement using Balances pallet or create custom token pallet
// See references/substrate-development.md for complete implementation
```

**Solana:**
```rust
// Use SPL Token or create custom token with Anchor
// See references/solana-development.md for implementation
```

### Implementing Governance

Design and implement on-chain governance mechanisms:
- Proposal submission and voting
- Timelock for proposal execution
- Quorum requirements
- Vote delegation

**See references for complete patterns and examples.**

### Building a DEX/AMM

Implement automated market maker with:
- Liquidity pool management
- Swap functionality with slippage protection
- Fee collection and distribution
- Price oracle integration

### Creating NFT Marketplace

Features typically include:
- Minting and metadata storage
- Listing and delisting
- Bidding and auction mechanisms
- Royalty enforcement

## Best Practices

### Code Organization

```rust
// Organize code logically
my-blockchain-project/
├── pallets/              // Substrate pallets
│   ├── profiles/
│   ├── governance/
│   └── tokens/
├── programs/             // Solana programs
│   ├── marketplace/
│   └── staking/
├── runtime/              // Substrate runtime
├── tests/               // Integration tests
└── scripts/             // Deployment scripts
```

### Performance Optimization

- Use bounded collections (BoundedVec, BoundedBTreeMap)
- Minimize storage reads/writes
- Batch operations when possible
- Profile and benchmark critical paths
- Use zero-copy for large accounts (Solana)

### Documentation

Document thoroughly:
- Public API documentation
- Security considerations
- Deployment procedures
- Upgrade mechanisms
- Emergency procedures

## Troubleshooting

### Common Issues

**Build failures:**
- Check Rust toolchain version
- Ensure WASM target installed
- Verify dependency versions

**Transaction failures:**
- Verify account has sufficient balance
- Check transaction weight/gas limits
- Validate input parameters
- Review error logs

**Security vulnerabilities:**
- Run `cargo audit` regularly
- Use checked arithmetic
- Validate all inputs
- Follow security checklist

## Reference Documentation

This skill includes comprehensive reference documentation:

### references/setup-environment.md
Complete setup instructions for all platforms, including:
- Rust installation and configuration
- Platform-specific dependencies
- Development tools
- IDE setup
- Testing environment

### references/substrate-development.md
Deep dive into Substrate framework:
- Pallet architecture
- Storage patterns
- Runtime development
- Testing strategies
- Benchmarking

### references/solana-development.md
Comprehensive Solana/Anchor guide:
- Program structure
- Account model
- PDAs and CPIs
- Anchor framework
- Testing

### references/security-best-practices.md
Security fundamentals:
- Common vulnerabilities
- Prevention techniques
- Access control patterns
- Audit checklist

### references/smart-contract-patterns.md
Design patterns for blockchain:
- Structural patterns
- Behavioral patterns
- Security patterns
- Economic patterns

### references/testing-deployment.md
Testing and deployment strategies:
- Testing approaches
- Deployment workflows
- Monitoring
- Maintenance

### references/system-architecture.md
System design and architecture:
- Architectural patterns
- Scalability solutions
- Network design
- Production setup

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/stable/)
- [Substrate Developer Hub](https://substrate.io/developers/)
- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/)

## License

MIT License - See LICENSE.txt for details
