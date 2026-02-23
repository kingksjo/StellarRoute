use crate::errors::ContractError;
use crate::events;
use crate::storage::{self, extend_instance_ttl, StorageKey};
use soroban_sdk::{contract, contractimpl, Address, Env};

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
            return Err(ContractError::PoolNotSupported); // Or a specific AlreadyRegistered error
        }

        e.storage().persistent().set(&key, &true);
        // Extend persistent storage TTL (Crucial!)
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

    // Guard function to be used in swap logic later
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
}
