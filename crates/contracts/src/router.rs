use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    self, extend_instance_ttl, get_fee_rate, get_fee_to, increment_nonce, is_supported_pool,
    transfer_asset, StorageKey,
};
use crate::types::{CommitmentData, MevConfig, QuoteResult, Route, SwapParams, SwapResult};
use soroban_sdk::{
    contract, contractimpl, symbol_short, vec, Address, Bytes, BytesN, Env, IntoVal, Symbol,
};

const CONTRACT_VERSION: u32 = 2;

#[contract]
pub struct StellarRoute;

#[contractimpl]
impl StellarRoute {
    pub fn initialize(
        e: Env,
        admin: Address,
        fee_rate: u32,
        fee_to: Address,
    ) -> Result<(), ContractError> {
        if e.storage().instance().has(&StorageKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        if fee_rate > 1000 {
            return Err(ContractError::InvalidAmount);
        }

        e.storage().instance().set(&StorageKey::Admin, &admin);
        e.storage().instance().set(&StorageKey::FeeRate, &fee_rate);
        e.storage().instance().set(&StorageKey::FeeTo, &fee_to);
        e.storage().instance().set(&StorageKey::Paused, &false);

        events::initialized(&e, admin, fee_rate);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn set_admin(e: Env, new_admin: Address) -> Result<(), ContractError> {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.storage().instance().set(&StorageKey::Admin, &new_admin);
        events::admin_changed(&e, admin, new_admin);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn register_pool(e: Env, pool: Address) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();

        let key = StorageKey::SupportedPool(pool.clone());
        if e.storage().persistent().has(&key) {
            return Err(ContractError::PoolNotSupported);
        }

        e.storage().persistent().set(&key, &true);
        e.storage().persistent().extend_ttl(&key, 17280, 17280 * 30);

        let new_count = storage::get_pool_count(&e) + 1;
        storage::set_pool_count(&e, new_count);

        events::pool_registered(&e, pool);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn pause(e: Env) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();
        e.storage().instance().set(&StorageKey::Paused, &true);
        events::paused(&e);
        Ok(())
    }

    pub fn unpause(e: Env) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();
        e.storage().instance().set(&StorageKey::Paused, &false);
        events::unpaused(&e);
        Ok(())
    }

    // --- Read-only getters for deployment verification and monitoring ---

    pub fn version(_e: Env) -> u32 {
        CONTRACT_VERSION
    }

    pub fn get_admin(e: Env) -> Result<Address, ContractError> {
        if !storage::is_initialized(&e) {
            return Err(ContractError::NotInitialized);
        }
        Ok(storage::get_admin(&e))
    }

    pub fn get_fee_rate_value(e: Env) -> u32 {
        storage::get_fee_rate(&e)
    }

    pub fn get_fee_to_address(e: Env) -> Result<Address, ContractError> {
        storage::get_fee_to_optional(&e).ok_or(ContractError::NotInitialized)
    }

    pub fn is_paused(e: Env) -> bool {
        storage::get_paused(&e)
    }

    pub fn get_pool_count(e: Env) -> u32 {
        storage::get_pool_count(&e)
    }

    pub fn is_pool_registered(e: Env, pool: Address) -> bool {
        storage::is_supported_pool(&e, pool)
    }

    // --- Admin MEV Configuration ---

    pub fn configure_mev(e: Env, config: MevConfig) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();
        storage::set_mev_config(&e, &config);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn set_whitelist(
        e: Env,
        address: Address,
        whitelisted: bool,
    ) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();
        storage::set_whitelisted(&e, &address, whitelisted);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn update_known_price(
        e: Env,
        token_a: Address,
        token_b: Address,
        price: i128,
    ) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();
        storage::set_latest_known_price(&e, &token_a, &token_b, price);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn get_mev_config(e: Env) -> Result<MevConfig, ContractError> {
        storage::get_mev_config(&e).ok_or(ContractError::NotInitialized)
    }

    // --- Commit-Reveal Pattern ---

    pub fn commit_swap(
        e: Env,
        sender: Address,
        commitment_hash: BytesN<32>,
        deposit_amount: i128,
    ) -> Result<(), ContractError> {
        sender.require_auth();
        StellarRoute::require_not_paused(&e)?;

        if deposit_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let mev_config = storage::get_mev_config(&e).ok_or(ContractError::NotInitialized)?;

        let current_ledger = e.ledger().sequence();
        let expires_at = current_ledger + mev_config.commit_window_ledgers;

        let commitment = CommitmentData {
            sender: sender.clone(),
            deposit_amount,
            created_at: current_ledger,
            expires_at,
        };

        storage::set_commitment(&e, &commitment_hash, &commitment, mev_config.commit_window_ledgers);

        events::commitment_created(&e, sender, commitment_hash, deposit_amount);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn reveal_and_execute(
        e: Env,
        sender: Address,
        params: SwapParams,
        salt: BytesN<32>,
    ) -> Result<SwapResult, ContractError> {
        sender.require_auth();
        StellarRoute::require_not_paused(&e)?;

        // Recompute hash from params + salt
        let mut payload = Bytes::new(&e);
        payload.append(&Bytes::from_slice(&e, &params.amount_in.to_be_bytes()));
        payload.append(&Bytes::from_slice(&e, &params.min_amount_out.to_be_bytes()));
        payload.append(&Bytes::from_slice(&e, &params.deadline.to_be_bytes()));
        let salt_bytes: Bytes = salt.into();
        payload.append(&salt_bytes);
        let computed_hash = e.crypto().sha256(&payload);

        // Verify commitment exists
        let commitment =
            storage::get_commitment(&e, &computed_hash).ok_or(ContractError::CommitmentNotFound)?;

        // Verify sender matches
        if commitment.sender != sender {
            return Err(ContractError::InvalidReveal);
        }

        // Verify not expired
        if e.ledger().sequence() > commitment.expires_at {
            return Err(ContractError::CommitmentExpired);
        }

        // Remove commitment
        storage::remove_commitment(&e, &computed_hash);

        events::commitment_revealed(&e, sender.clone(), computed_hash);

        // Execute the swap using the internal logic
        Self::execute_swap_internal(&e, &sender, &params)
    }

    // --- Core operations ---

    pub fn require_not_paused(e: &Env) -> Result<(), ContractError> {
        let paused: bool = e
            .storage()
            .instance()
            .get(&StorageKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::Paused);
        }
        Ok(())
    }

    /// Public entry point for users to get quotes
    pub fn get_quote(e: Env, amount_in: i128, route: Route) -> Result<QuoteResult, ContractError> {
        if amount_in <= 0 || route.hops.is_empty() || route.hops.len() > 4 {
            return Err(ContractError::InvalidRoute);
        }

        let mut current_amount = amount_in;
        let mut total_impact_bps: u32 = 0;

        for i in 0..route.hops.len() {
            let hop = route.hops.get(i).unwrap();
            if !is_supported_pool(&e, hop.pool.clone()) {
                return Err(ContractError::PoolNotSupported);
            }

            let call_result = e.try_invoke_contract::<i128, soroban_sdk::Error>(
                &hop.pool,
                &Symbol::new(&e, "adapter_quote"),
                vec![
                    &e,
                    hop.source.into_val(&e),
                    hop.destination.into_val(&e),
                    current_amount.into_val(&e),
                ],
            );

            current_amount = match call_result {
                Ok(Ok(val)) => val,
                _ => return Err(ContractError::PoolCallFailed),
            };
            total_impact_bps += 5;
        }

        let fee_rate = get_fee_rate(&e);
        let fee_amount = (current_amount * fee_rate as i128) / 10000;
        let final_output = current_amount - fee_amount;

        Ok(QuoteResult {
            expected_output: final_output,
            price_impact_bps: total_impact_bps,
            fee_amount,
            route: route.clone(),
            valid_until: (e.ledger().sequence() + 120) as u64,
        })
    }

    pub fn execute_swap(
        e: Env,
        sender: Address,
        params: SwapParams,
    ) -> Result<SwapResult, ContractError> {
        sender.require_auth();
        StellarRoute::require_not_paused(&e)?;

        // Check commit-reveal requirement for large swaps
        if let Some(mev_config) = storage::get_mev_config(&e) {
            if params.amount_in >= mev_config.commit_threshold {
                return Err(ContractError::CommitmentRequired);
            }
        }

        Self::execute_swap_internal(&e, &sender, &params)
    }

    // --- Internal swap execution (shared by execute_swap and reveal_and_execute) ---

    fn execute_swap_internal(
        e: &Env,
        sender: &Address,
        params: &SwapParams,
    ) -> Result<SwapResult, ContractError> {
        // 1. Deadline check
        if e.ledger().sequence() as u64 > params.deadline {
            return Err(ContractError::DeadlineExceeded);
        }

        // 2. Not-before check
        if (e.ledger().sequence() as u64) < params.not_before {
            return Err(ContractError::ExecutionTooEarly);
        }

        // 3. Route validation
        if params.route.hops.is_empty() || params.route.hops.len() > 4 {
            return Err(ContractError::InvalidRoute);
        }

        // 4. Rate limiting (if MEV config is set)
        if let Some(mev_config) = storage::get_mev_config(e) {
            if !storage::is_whitelisted(e, sender) {
                let current_ledger = e.ledger().sequence();
                let window_start = storage::get_account_swap_window_start(e, sender);
                let swap_count = storage::get_account_swap_count(e, sender);

                if window_start > 0
                    && current_ledger < window_start + mev_config.rate_limit_window
                {
                    // Still within the window
                    if swap_count >= mev_config.max_swaps_per_window {
                        events::rate_limit_hit(
                            e,
                            sender.clone(),
                            swap_count,
                            mev_config.rate_limit_window,
                        );
                        return Err(ContractError::RateLimitExceeded);
                    }
                    storage::set_account_swap_count(
                        e,
                        sender,
                        swap_count + 1,
                        mev_config.rate_limit_window,
                    );
                } else {
                    // Window expired or first swap — reset
                    storage::set_account_swap_window_start(
                        e,
                        sender,
                        current_ledger,
                        mev_config.rate_limit_window,
                    );
                    storage::set_account_swap_count(e, sender, 1, mev_config.rate_limit_window);
                }
            }
        }

        // 5. Snapshot pool reserves before swap (for sandwich detection)
        let mut pre_reserves: soroban_sdk::Vec<(i128, i128)> = soroban_sdk::Vec::new(e);
        for i in 0..params.route.hops.len() {
            let hop = params.route.hops.get(i).unwrap();
            let reserves_result = e.try_invoke_contract::<(i128, i128), soroban_sdk::Error>(
                &hop.pool,
                &symbol_short!("get_rsrvs"),
                vec![e],
            );
            let reserves = match reserves_result {
                Ok(Ok(val)) => val,
                _ => (0_i128, 0_i128), // If pool doesn't support reserves, skip check
            };
            pre_reserves.push_back(reserves);
        }

        // 6. Transfer input to first pool
        let mut current_input_amount = params.amount_in;
        let first_hop = params.route.hops.get(0).unwrap();
        transfer_asset(
            e,
            &first_hop.source,
            sender,
            &first_hop.pool,
            params.amount_in,
        );

        // 7. Execute swap hops
        let mut total_impact_bps: u32 = 0;
        for i in 0..params.route.hops.len() {
            let hop = params.route.hops.get(i).unwrap();

            if !is_supported_pool(e, hop.pool.clone()) {
                return Err(ContractError::PoolNotSupported);
            }

            let call_result = e.try_invoke_contract::<i128, soroban_sdk::Error>(
                &hop.pool,
                &symbol_short!("swap"),
                vec![
                    e,
                    hop.source.into_val(e),
                    hop.destination.into_val(e),
                    current_input_amount.into_val(e),
                    0_i128.into_val(e),
                ],
            );

            current_input_amount = match call_result {
                Ok(Ok(val)) => val,
                _ => return Err(ContractError::PoolCallFailed),
            };
            total_impact_bps += 5;
        }

        // 8. Calculate fees
        let fee_rate = get_fee_rate(e);
        let fee_amount = (current_input_amount * fee_rate as i128) / 10000;
        let final_output = current_input_amount - fee_amount;

        // 9. Enhanced slippage guards
        // max_price_impact_bps check
        if params.max_price_impact_bps > 0 && total_impact_bps > params.max_price_impact_bps {
            return Err(ContractError::PriceImpactTooHigh);
        }

        // max_execution_spread_bps check (compare actual output vs expected)
        if params.max_execution_spread_bps > 0 && params.route.estimated_output > 0 {
            let spread = if final_output < params.route.estimated_output {
                ((params.route.estimated_output - final_output) * 10000)
                    / params.route.estimated_output
            } else {
                0
            };
            if spread > params.max_execution_spread_bps as i128 {
                return Err(ContractError::SpreadTooHigh);
            }
        }

        // Standard slippage check
        if final_output < params.min_amount_out {
            return Err(ContractError::SlippageExceeded);
        }

        // 10. Post-swap reserve validation (sandwich detection)
        for i in 0..params.route.hops.len() {
            let hop = params.route.hops.get(i).unwrap();
            let pre = pre_reserves.get(i).unwrap();
            if pre.0 == 0 && pre.1 == 0 {
                continue; // Skip if pre-snapshot wasn't available
            }

            let post_result = e.try_invoke_contract::<(i128, i128), soroban_sdk::Error>(
                &hop.pool,
                &symbol_short!("get_rsrvs"),
                vec![e],
            );
            if let Ok(Ok(post)) = post_result {
                // Check that reserves changed in the expected direction
                // For a swap: one reserve goes up, one goes down
                let delta_0 = post.0 - pre.0;
                let delta_1 = post.1 - pre.1;
                // If both reserves moved in the same direction, something is wrong
                if delta_0 > 0 && delta_1 > 0 {
                    return Err(ContractError::ReserveManipulationDetected);
                }
                if delta_0 < 0 && delta_1 < 0 {
                    return Err(ContractError::ReserveManipulationDetected);
                }
            }
        }

        // 11. Emit high impact event if configured
        if let Some(mev_config) = storage::get_mev_config(e) {
            if total_impact_bps > mev_config.high_impact_threshold_bps {
                events::high_impact_swap(e, sender.clone(), total_impact_bps, params.amount_in);
            }
        }

        // 12. Transfer output to recipient
        let last_hop = params.route.hops.get(params.route.hops.len() - 1).unwrap();

        transfer_asset(
            e,
            &last_hop.destination,
            &e.current_contract_address(),
            &params.recipient,
            final_output,
        );

        transfer_asset(
            e,
            &last_hop.destination,
            &e.current_contract_address(),
            &get_fee_to(e),
            fee_amount,
        );

        increment_nonce(e, sender.clone());

        events::swap_executed(
            e,
            sender.clone(),
            params.amount_in,
            final_output,
            fee_amount,
            params.route.clone(),
        );

        Ok(SwapResult {
            amount_in: params.amount_in,
            amount_out: final_output,
            route: params.route.clone(),
            executed_at: e.ledger().sequence() as u64,
        })
    }
}
