use crate::errors::ContractError;
use crate::storage::{get_admin, is_paused, set_admin, StorageKey};
use crate::types::SwapParams;
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct StellarRoute;

#[contractimpl]
impl StellarRoute {
    pub fn init(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&StorageKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        set_admin(&env, &admin);
        Ok(())
    }
    pub fn swap(env: Env, params: SwapParams) -> Result<(), ContractError> {
        // Use the admin check just to clear the warning
        let _admin = get_admin(&env);

        if is_paused(&env) {
            return Err(ContractError::Paused);
        }

        // Use the route from params to clear the warning
        if params.route.hops.is_empty() {
            return Err(ContractError::EmptyRoute);
        }

        if env.ledger().timestamp() >= params.deadline {
            return Err(ContractError::DeadlineExceeded);
        }

        Ok(())
    }
}
