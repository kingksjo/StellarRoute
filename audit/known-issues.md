# Known Issues & Accepted Risks

## Known Limitations

### 1. Single Admin Model
- **Description**: The contract uses a single admin address for all privileged operations. There is no multi-sig, timelock, or role-based access control.
- **Risk**: Admin key compromise grants full control.
- **Mitigation**: Operational key management policy (see `docs/deployment/README.md`). Multi-sig can be implemented at the account level using Stellar's native multi-sig features.
- **Status**: Accepted for MVP. Multi-sig governance planned for future milestone.

### 2. No Pause Check in `get_quote()`
- **Description**: The `get_quote()` function does not check `is_paused` before executing. A paused contract still returns quotes (but actual swaps would fail at the pool level if a swap function existed with pause checks).
- **Risk**: Users may receive quotes that cannot be executed.
- **Mitigation**: Front-end and SDK should check `is_paused()` before presenting quotes.
- **Status**: Accepted. Adding a pause check to `get_quote()` is a future enhancement.

### 3. Pool Enumeration Not Supported
- **Description**: There is no function to list all registered pool addresses. Only individual pool checks via `is_pool_registered()` are supported.
- **Risk**: Off-chain systems must maintain their own pool registry.
- **Mitigation**: Pool registration events (`reg_pool`) can be indexed to reconstruct the full list.
- **Status**: Accepted. Soroban persistent storage does not support iteration.

### 4. `register_pool()` Error Name Mismatch
- **Description**: When a pool is already registered, the function returns `PoolNotSupported` instead of a more descriptive error like `PoolAlreadyRegistered`.
- **Risk**: Confusing error message for integrators.
- **Mitigation**: Error code (30) is documented; a rename is a non-breaking change.
- **Status**: Known. Will be fixed in a future cleanup PR.

### 5. Storage TTL Expiry Risk
- **Description**: If instance or persistent storage TTLs are not extended before expiry, contract state is permanently lost.
- **Risk**: Contract becomes non-functional and requires redeployment.
- **Mitigation**: TTLs are extended on every write operation. Monitoring script (`scripts/monitor.sh`) checks for TTL health. Recommended to run monitoring on a schedule.
- **Status**: Accepted. Operational risk managed via monitoring.

### 6. Fixed Price Impact Estimate
- **Description**: Price impact is currently a fixed 5 bps per hop rather than a calculated value based on actual liquidity depth.
- **Risk**: Inaccurate price impact reporting to users.
- **Mitigation**: Documented as an estimate. Accurate calculation planned for M2 (Soroban AMM integration).
- **Status**: Accepted for M1.

### 7. No Contract Upgrade Mechanism in Code
- **Description**: The contract does not include an explicit `upgrade()` function. Upgrades rely on Soroban's native WASM replacement mechanism via CLI.
- **Risk**: Upgrade process is CLI-dependent and not programmatically gated.
- **Mitigation**: Upgrade script (`scripts/upgrade.sh`) captures pre/post state and verifies invariants. Admin-only access via Soroban CLI identity.
- **Status**: Accepted. Native Soroban upgrade path is sufficient for current stage.
