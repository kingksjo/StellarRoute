# StellarRoute Findings & Research

**Purpose:** Store research discoveries, technical findings, and important information gathered during development.

---

## Stellar/Soroban Research

### Stellar Horizon API
- **Status:** Need to research
- **Notes:** Need to investigate endpoints for SDEX orderbook data
- **Links:** TBD

### Soroban Development
- **Status:** Need to research
- **Notes:** 
  - Soroban is Stellar's smart contract platform
  - Uses Rust SDK
  - Need to understand AMM contract interfaces
- **Links:** 
  - https://developers.stellar.org/docs/build/smart-contracts/overview
  - https://developers.stellar.org/docs/tools/sdks/contract-sdks

---

## Technology Stack Decisions

### Backend Framework
- **Candidate:** Axum or Actix-web
- **Decision:** Pending research
- **Reasoning:** Need to evaluate performance, ecosystem, and Rust async support

### Database ORM
- **Candidates:** sqlx or diesel
- **Decision:** Pending research
- **Reasoning:** Need to evaluate type safety, async support, and migration capabilities

---

## SDK/Library Discoveries

### Rust Stellar SDK
- **Status:** Need to verify
- **Package:** rust-stellar-sdk (verify actual package name)
- **Notes:** Need to find official Rust SDK for Stellar

---

## Key Insights

- Stellar uses WASM for smart contracts (Soroban)
- Need to support both SDEX (orderbook) and Soroban AMM pools
- Performance target: <500ms API latency

---

## Open Questions

1. What is the official Rust SDK for Stellar Horizon API?
2. What are the existing Soroban AMM contract interfaces?
3. What are the best practices for indexing Stellar orderbooks in real-time?

## Environment Setup Notes

### Rust Installation
- **Issue:** SSL connection error when attempting automated Rust installation
- **Resolution:** Need manual Rust installation or verify network connectivity
- **Manual Installation Command:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **After Installation:** Run `rustup target add wasm32-unknown-unknown` for Soroban support

---

## Notes

- Update this file after every research/discovery session
- Include links and references
- Note important technical details
