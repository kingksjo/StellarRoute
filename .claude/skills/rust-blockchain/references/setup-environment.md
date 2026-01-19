# Rust Blockchain Development - Setup & Environment

## Table of Contents
- [Prerequisites](#prerequisites)
- [Rust Installation & Configuration](#rust-installation--configuration)
- [Development Tools](#development-tools)
- [Platform-Specific Setup](#platform-specific-setup)
- [Testing Environment](#testing-environment)

## Prerequisites

### System Requirements
- **OS**: Linux (recommended), macOS, or Windows (WSL2 recommended)
- **RAM**: Minimum 8GB, recommended 16GB+
- **Storage**: 50GB+ free space for blockchain nodes
- **CPU**: Multi-core processor (4+ cores recommended)

### Required Software
```bash
# Install Rust via rustup (always use latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure current shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## Rust Installation & Configuration

### Rustup Configuration
```bash
# Set default toolchain to stable
rustup default stable

# Add nightly for certain blockchain features
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# Add wasm target for smart contracts
rustup target add wasm32-unknown-unknown

# Update regularly
rustup update
```

### Cargo Configuration
Create or edit `~/.cargo/config.toml`:

```toml
[build]
# Optimize for blockchain applications
rustflags = ["-C", "link-arg=-s"]

[target.wasm32-unknown-unknown]
# WASM-specific optimizations
rustflags = [
    "-C", "link-arg=--import-memory",
    "-C", "link-arg=--export-table",
]

[net]
git-fetch-with-cli = true

[alias]
# Useful aliases for blockchain development
contract-build = "build --release --target wasm32-unknown-unknown"
test-all = "test --all-features"
```

## Development Tools

### Essential CLI Tools
```bash
# Install cargo-edit (manage dependencies)
cargo install cargo-edit

# Install cargo-watch (auto-rebuild on changes)
cargo install cargo-watch

# Install cargo-audit (security vulnerabilities)
cargo install cargo-audit

# Install cargo-expand (macro debugging)
cargo install cargo-expand

# Install wasm-pack (WebAssembly tooling)
cargo install wasm-pack

# Install cargo-contract (ink! smart contracts)
cargo install cargo-contract --force

# Install dylint (custom linting)
cargo install cargo-dylint dylint-link
```

### Substrate Development
```bash
# Install Substrate dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install -y git clang curl libssl-dev llvm libudev-dev \
    make protobuf-compiler pkg-config

# macOS dependencies
brew install openssl cmake llvm protobuf

# Install Substrate node template
cargo install node-template --git https://github.com/substrate-developer-hub/substrate-node-template

# Install subxt (interact with Substrate chains)
cargo install subxt-cli
```

### Solana Development
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify installation
solana --version

# Install Anchor framework (recommended for Solana)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Verify Anchor
anchor --version
```

### NEAR Protocol
```bash
# Install NEAR CLI
npm install -g near-cli

# Install Rust contract dependencies
rustup target add wasm32-unknown-unknown
```

### CosmWasm (Cosmos Ecosystem)
```bash
# Install cargo-generate for templates
cargo install cargo-generate

# Install cosmwasm-check for contract validation
cargo install cosmwasm-check

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

## Platform-Specific Setup

### Substrate Project Initialization
```bash
# Clone Substrate node template
git clone https://github.com/substrate-developer-hub/substrate-node-template
cd substrate-node-template

# Build the node
cargo build --release

# Run development node
./target/release/node-template --dev

# In another terminal, start frontend
cd ../substrate-front-end-template
yarn install
yarn start
```

### Solana Project Initialization
```bash
# Create new Anchor project
anchor init my-solana-project
cd my-solana-project

# Project structure:
# ├── Anchor.toml          # Anchor configuration
# ├── Cargo.toml          # Workspace configuration
# ├── programs/           # Smart contracts
# ├── tests/              # TypeScript tests
# └── migrations/         # Deployment scripts

# Configure Solana CLI for development
solana config set --url localhost
solana-test-validator  # Start local validator

# Build and deploy
anchor build
anchor deploy

# Run tests
anchor test
```

### NEAR Project Initialization
```bash
# Create new NEAR project
cargo generate --git https://github.com/near/near-sdk-rs --name my-near-project
cd my-near-project

# Build contract
cargo build --target wasm32-unknown-unknown --release

# Test contract
cargo test
```

### CosmWasm Project Initialization
```bash
# Generate from template
cargo generate --git https://github.com/CosmWasm/cw-template.git --name my-cosmwasm-project
cd my-cosmwasm-project

# Build contract
cargo wasm

# Run tests
cargo test

# Optimize WASM binary
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13
```

## Testing Environment

### Unit Testing Setup
```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.0"          # Property-based testing
criterion = "0.5"         # Benchmarking
mockall = "0.11"          # Mocking
tokio-test = "0.4"        # Async testing
```

### Integration Testing Environment
```bash
# Install Docker for node testing
docker pull parity/substrate:latest
docker pull solanalabs/solana:latest

# Create test scripts
mkdir -p scripts/test
```

### Benchmarking Setup
Create `benches/` directory:

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Your blockchain operation
            black_box(my_function())
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## Environment Variables

Create `.env` file for development:

```bash
# Network configuration
NETWORK=devnet  # or testnet, mainnet
RPC_URL=http://localhost:8899

# Substrate specific
NODE_URL=ws://localhost:9944
SUBSTRATE_VERSION=3.0.0

# Solana specific
SOLANA_PROGRAM_ID=YourProgramId
ANCHOR_PROVIDER_URL=http://localhost:8899
ANCHOR_WALLET=~/.config/solana/id.json

# NEAR specific
NEAR_ACCOUNT_ID=your-account.testnet
NEAR_NETWORK=testnet

# Security (never commit these!)
PRIVATE_KEY=
MNEMONIC_PHRASE=

# Build optimization
RUSTFLAGS="-C target-cpu=native"
CARGO_BUILD_JOBS=4
```

## IDE Configuration

### VS Code Settings
Install extensions:
- rust-analyzer
- CodeLLDB (debugging)
- Even Better TOML
- Substrate Extension
- Solana Extension

Create `.vscode/settings.json`:

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.allTargets": true,
  "editor.formatOnSave": true,
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.procMacro.enable": true
}
```

### Debugging Configuration
`.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Substrate Node",
      "cargo": {
        "args": ["build", "--bin=node-template"],
        "filter": {
          "name": "node-template",
          "kind": "bin"
        }
      },
      "args": ["--dev"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## Verification

Run these commands to verify your setup:

```bash
# Check Rust installation
rustc --version
cargo --version
rustup show

# Check targets
rustup target list --installed

# Check tools
cargo --list | grep -E "contract|wasm|audit|watch"

# Platform-specific checks
substrate --version 2>/dev/null || echo "Substrate not installed"
solana --version 2>/dev/null || echo "Solana not installed"
anchor --version 2>/dev/null || echo "Anchor not installed"
near --version 2>/dev/null || echo "NEAR CLI not installed"

# Build a simple test project
cargo new --lib test-blockchain
cd test-blockchain
cargo build
cargo test
```

## Common Issues & Solutions

### Issue: WASM compilation fails
```bash
# Solution: Reinstall wasm target
rustup target remove wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup target add wasm32-unknown-unknown --toolchain stable
```

### Issue: OpenSSL errors on macOS
```bash
# Solution: Set OpenSSL paths
brew install openssl@1.1
export OPENSSL_DIR=$(brew --prefix openssl@1.1)
export OPENSSL_LIB_DIR=$(brew --prefix openssl@1.1)/lib
export OPENSSL_INCLUDE_DIR=$(brew --prefix openssl@1.1)/include
```

### Issue: Protobuf compiler not found
```bash
# Ubuntu/Debian
sudo apt install protobuf-compiler

# macOS
brew install protobuf

# Verify
protoc --version
```

### Issue: Linker errors on Windows
```bash
# Install Visual Studio Build Tools
# Or use WSL2 (recommended for blockchain development)
wsl --install
```

## Performance Optimization

### Build Performance
```toml
# .cargo/config.toml
[build]
# Use faster linker
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[profile.dev]
# Faster compilation for development
opt-level = 0
debug = true

[profile.release]
# Maximum optimization for production
opt-level = 3
lto = true
codegen-units = 1
```

### Caching
```bash
# Use sccache for faster rebuilds
cargo install sccache

# Configure in ~/.cargo/config.toml
[build]
rustc-wrapper = "sccache"
```

## Security Setup

### Audit Dependencies
```bash
# Run cargo audit regularly
cargo audit

# Set up pre-commit hook
echo "cargo audit" > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Secure Key Management
```bash
# Use hardware wallets for mainnet
# Never commit private keys
echo "*.key" >> .gitignore
echo ".env" >> .gitignore
echo "secrets/" >> .gitignore
```

This setup provides a solid foundation for professional Rust blockchain development across multiple platforms and frameworks.
