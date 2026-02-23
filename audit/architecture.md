# Contract Architecture

## Overview

StellarRoute is a DEX aggregation router deployed as a single Soroban smart contract on the Stellar network. It aggregates liquidity across SDEX orderbooks and Soroban AMM pools to provide optimal trade routing.

## Contract Structure

```
crates/contracts/src/
├── lib.rs        # Module declarations and contract export
├── router.rs     # Main contract logic (all public entrypoints)
├── storage.rs    # Storage key definitions and helper functions
├── types.rs      # Data types (Asset, Route, RouteHop, QuoteResult, etc.)
├── errors.rs     # Typed error enum (ContractError)
├── events.rs     # Event emission helpers
└── test.rs       # Unit tests
```

## Data Flow

### Initialization
```
Admin -> initialize(admin, fee_rate, fee_to)
  -> Store admin address (instance storage)
  -> Store fee configuration (instance storage)
  -> Set paused = false
  -> Emit "init" event
  -> Extend instance TTL
```

### Quote Request
```
User -> get_quote(amount_in, route)
  -> Validate inputs (amount > 0, hops <= 4, hops not empty)
  -> For each hop:
     -> Verify pool is registered (persistent storage lookup)
     -> Invoke pool's swap_out() to get output amount
  -> Calculate fee (fee_rate / 10000 of final output)
  -> Return QuoteResult with expected output, impact, fee, validity
```

### Pool Registration
```
Admin -> register_pool(pool_address)
  -> Verify caller is admin (require_auth)
  -> Check pool not already registered
  -> Store pool in persistent storage with 30-day TTL
  -> Increment pool counter
  -> Emit "reg_pool" event
```

## Trust Model

### Trusted Entities
- **Admin**: Single address with full control over contract configuration, pool registration, and emergency pause. Set during initialization, transferable via `set_admin()`.

### Trust Boundaries
- **External pool contracts**: The router invokes `swap_out()` on registered pools. A malicious pool could return incorrect amounts. Mitigation: only admin-registered pools are callable.
- **Soroban runtime**: Assumed correct and secure.
- **Horizon/RPC**: Off-chain data sources for the indexer; not part of the contract trust model.

### Threat Model
1. **Malicious admin**: Admin can pause, change fees, register malicious pools. Mitigation: transparent on-chain governance, admin key management policy.
2. **Malicious pool**: Registered pool returns inflated output. Mitigation: admin vetting before registration; price impact calculation provides user warning.
3. **Front-running**: Soroban's execution model processes transactions sequentially within a ledger. Standard MEV concerns are reduced but not eliminated.
4. **Storage expiry**: If instance or persistent storage TTLs expire, contract state is lost. Mitigation: TTL extension on every write operation.
