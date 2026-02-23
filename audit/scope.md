# Audit Scope

## In-Scope Files

All files under `crates/contracts/src/`:

| File | Lines | Description | Priority |
|------|-------|-------------|----------|
| `router.rs` | ~175 | All public entrypoints, core routing logic | Critical |
| `storage.rs` | ~70 | Storage key enum, read/write helpers, TTL management | Critical |
| `types.rs` | ~60 | Data structures for routes, quotes, assets | High |
| `errors.rs` | ~25 | Error enum with u32 codes | Medium |
| `events.rs` | ~25 | Event emission for state changes | Medium |
| `lib.rs` | ~12 | Module declarations | Low |

## In-Scope Functions (Critical Path)

### State-Changing (Admin-Gated)
- `initialize(admin, fee_rate, fee_to)` — one-time setup
- `set_admin(new_admin)` — admin transfer
- `register_pool(pool)` — whitelist a liquidity pool
- `pause()` / `unpause()` — emergency circuit breaker

### State-Changing (User-Facing)
- `get_quote(amount_in, route)` — cross-contract calls to pools

### Read-Only (Verification/Monitoring)
- `version()`, `get_admin()`, `get_fee_rate_value()`, `get_fee_to()`
- `is_paused()`, `get_pool_count()`, `is_pool_registered(pool)`

## Out of Scope

- `crates/indexer/` — off-chain indexing service
- `crates/api/` — REST API server
- `crates/routing/` — off-chain pathfinding algorithms
- `crates/sdk-rust/` — Rust SDK client
- `frontend/` — Web UI
- `scripts/` — Deployment tooling (operational, not security-critical)

## Key Areas to Focus

1. **Access control**: Every admin function must check `require_auth()` against stored admin.
2. **Initialization guard**: `initialize()` must only succeed once.
3. **Cross-contract calls**: `get_quote()` invokes external pool contracts; verify error handling.
4. **Arithmetic safety**: Fee calculation, price impact — check for overflow/underflow.
5. **Storage TTL management**: Verify TTLs are extended appropriately to prevent data loss.
6. **Input validation**: Route length limits, amount bounds, fee rate bounds.
