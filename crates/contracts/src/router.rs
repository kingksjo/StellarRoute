use crate::errors::ContractError;
use crate::events;
use crate::storage::{self, extend_instance_ttl, get_fee_rate, is_supported_pool, StorageKey};
use crate::types::{QuoteResult, Route};
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, IntoVal};

const CONTRACT_VERSION: u32 = 1;

#[contract]
pub struct StellarRoute;

#[contractimpl]
impl StellarRoute {
    /// Issue #38: Initialize the contract with admin and fee settings
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

    /// Issue #38: Transfer admin rights
    pub fn set_admin(e: Env, new_admin: Address) -> Result<(), ContractError> {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.storage().instance().set(&StorageKey::Admin, &new_admin);
        events::admin_changed(&e, admin, new_admin);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Issue #38: Register a liquidity pool
    pub fn register_pool(e: Env, pool: Address) -> Result<(), ContractError> {
        storage::get_admin(&e).require_auth();

        let key = StorageKey::SupportedPool(pool.clone());
        if e.storage().persistent().has(&key) {
            return Err(ContractError::PoolNotSupported);
        }

        e.storage().persistent().set(&key, &true);
        // Extend persistent storage TTL (approx 30 days)
        e.storage().persistent().extend_ttl(&key, 17280, 17280 * 30);

        let new_count = storage::get_pool_count(&e) + 1;
        storage::set_pool_count(&e, new_count);

        events::pool_registered(&e, pool);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Issue #38: Emergency Pause
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

    pub fn get_fee_to(e: Env) -> Result<Address, ContractError> {
        storage::get_fee_to(&e).ok_or(ContractError::NotInitialized)
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

    pub fn get_quote(e: Env, amount_in: i128, route: Route) -> Result<QuoteResult, ContractError> {
        if amount_in <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        if route.hops.is_empty() {
            return Err(ContractError::EmptyRoute);
        }
        if route.hops.len() > 4 {
            return Err(ContractError::TooManyHops);
        }

        let mut current_amount = amount_in;
        let mut total_impact_bps: u32 = 0;

        for i in 0..route.hops.len() {
            let hop = route.hops.get(i).unwrap();

            if !is_supported_pool(&e, hop.pool.clone()) {
                return Err(ContractError::PoolNotSupported);
            }

            // Fixed: Added soroban_sdk::Error as the second generic argument
            let call_result = e.try_invoke_contract::<i128, soroban_sdk::Error>(
                &hop.pool,
                &symbol_short!("swap_out"),
                vec![
                    &e,
                    hop.source.into_val(&e),
                    hop.destination.into_val(&e),
                    current_amount.into_val(&e),
                ],
            );

            let output_from_hop = match call_result {
                Ok(Ok(val)) => val,
                _ => return Err(ContractError::PoolCallFailed),
            };

            total_impact_bps += 5;
            current_amount = output_from_hop;
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
}
