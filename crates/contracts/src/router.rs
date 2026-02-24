use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    self, batch_check_pools, extend_instance_ttl, get_fee_rate, get_instance_config,
    increment_nonce, transfer_asset, StorageKey,
};
use crate::types::{QuoteResult, ResourceEstimate, Route, SwapParams, SwapResult};
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, IntoVal, Symbol, Vec};

const CONTRACT_VERSION: u32 = 1;
const MAX_HOPS: u32 = 4;
const BASE_CPU_PER_HOP: u64 = 5_000_000; // ~5M instructions per hop
const CCI_OVERHEAD: u64 = 2_000_000; // Cross-contract call overhead

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

    /// Estimate resource consumption for a swap operation
    pub fn estimate_resources(
        _e: Env,
        amount_in: i128,
        route: Route,
    ) -> Result<ResourceEstimate, ContractError> {
        if amount_in <= 0 || route.hops.is_empty() {
            return Err(ContractError::InvalidRoute);
        }

        let num_hops = route.hops.len() as u32;
        if num_hops > MAX_HOPS {
            return Err(ContractError::InvalidRoute);
        }

        // Estimate CPU: base + per-hop + CCI overhead
        let estimated_cpu = (BASE_CPU_PER_HOP * num_hops as u64) + (CCI_OVERHEAD * num_hops as u64);

        // Storage reads: 1 instance config + num_hops pool checks + 1 nonce
        let storage_reads = 1 + num_hops + 1;

        // Storage writes: 1 nonce update
        let storage_writes = 1;

        // Events: 1 swap event
        let events = 1;

        // Will succeed if under 100M instructions
        let will_succeed = estimated_cpu < 100_000_000;

        Ok(ResourceEstimate {
            estimated_cpu,
            storage_reads,
            storage_writes,
            events,
            will_succeed,
        })
    }

    /// Public entry point for users to get quotes
    pub fn get_quote(e: Env, amount_in: i128, route: Route) -> Result<QuoteResult, ContractError> {
        if amount_in <= 0 || route.hops.is_empty() || route.hops.len() > MAX_HOPS {
            return Err(ContractError::InvalidRoute);
        }

        // Pre-allocate with known capacity to avoid reallocation
        let mut pools = Vec::new(&e);
        for i in 0..route.hops.len() {
            pools.push_back(route.hops.get(i).unwrap().pool.clone());
        }

        // Batch check all pools at once
        if !batch_check_pools(&e, &pools) {
            return Err(ContractError::PoolNotSupported);
        }

        let mut current_amount = amount_in;
        let mut total_impact_bps: u32 = 0;

        for i in 0..route.hops.len() {
            let hop = route.hops.get(i).unwrap();

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

        // Batch read all instance config in one operation (optimization)
        let config = get_instance_config(&e);
        if config.paused {
            return Err(ContractError::Paused);
        }

        if e.ledger().sequence() as u64 > params.deadline {
            return Err(ContractError::DeadlineExceeded);
        }

        if params.route.hops.is_empty() || params.route.hops.len() > MAX_HOPS {
            return Err(ContractError::InvalidRoute);
        }

        // Pre-allocate and batch check pools
        let mut pools = Vec::new(&e);
        for i in 0..params.route.hops.len() {
            pools.push_back(params.route.hops.get(i).unwrap().pool.clone());
        }
        if !batch_check_pools(&e, &pools) {
            return Err(ContractError::PoolNotSupported);
        }

        let mut current_input_amount = params.amount_in;

        let first_hop = params.route.hops.get(0).unwrap();
        transfer_asset(
            &e,
            &first_hop.source,
            &sender,
            &first_hop.pool,
            params.amount_in,
        );

        for i in 0..params.route.hops.len() {
            let hop = params.route.hops.get(i).unwrap();

            let call_result = e.try_invoke_contract::<i128, soroban_sdk::Error>(
                &hop.pool,
                &symbol_short!("swap"),
                vec![
                    &e,
                    hop.source.into_val(&e),
                    hop.destination.into_val(&e),
                    current_input_amount.into_val(&e),
                    0_i128.into_val(&e),
                ],
            );

            current_input_amount = match call_result {
                Ok(Ok(val)) => val,
                _ => return Err(ContractError::PoolCallFailed),
            };
        }

        // Use cached fee_rate from config
        let fee_amount = (current_input_amount * config.fee_rate as i128) / 10000;
        let final_output = current_input_amount - fee_amount;

        if final_output < params.min_amount_out {
            return Err(ContractError::SlippageExceeded);
        }

        let last_hop = params.route.hops.get(params.route.hops.len() - 1).unwrap();

        transfer_asset(
            &e,
            &last_hop.destination,
            &e.current_contract_address(),
            &params.recipient,
            final_output,
        );

        transfer_asset(
            &e,
            &last_hop.destination,
            &e.current_contract_address(),
            &config.fee_to,
            fee_amount,
        );

        increment_nonce(&e, sender.clone());

        // Emit compact event (use IDs instead of full structs where possible)
        events::swap_executed(
            &e,
            sender,
            params.amount_in,
            final_output,
            fee_amount,
            params.route.clone(),
        );

        Ok(SwapResult {
            amount_in: params.amount_in,
            amount_out: final_output,
            route: params.route,
            executed_at: e.ledger().sequence() as u64,
        })
    }
}
