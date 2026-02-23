use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    self, extend_instance_ttl, get_fee_rate, get_fee_to, increment_nonce, is_supported_pool,
    transfer_asset, StorageKey,
};
use crate::types::{
    ContractVersion, GovernanceConfig, Proposal, ProposalAction, QuoteResult, Route, SwapParams,
    SwapResult, TokenCategory, TokenInfo,
};
use crate::{governance, tokens, upgrade};
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, BytesN, Env, IntoVal, Symbol, Vec};

#[contract]
pub struct StellarRoute;

#[contractimpl]
impl StellarRoute {
    /// Initialize the contract.
    ///
    /// When `signers` is non-empty the contract starts in multi-sig mode
    /// immediately. Otherwise it starts in single-admin mode and can be
    /// migrated later via `migrate_to_multisig`.
    pub fn initialize(
        e: Env,
        admin: Address,
        fee_rate: u32,
        fee_to: Address,
        // ── Optional multi-sig bootstrap ─────────────────────────────────────
        signers: Option<Vec<Address>>,
        threshold: Option<u32>,
        proposal_ttl: Option<u64>,
        guardian: Option<Address>,
        // ── Optional initial WASM hash for version tracking ──────────────────
        initial_wasm_hash: Option<BytesN<32>>,
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

        // Bootstrap multi-sig if signers provided.
        if let (Some(s), Some(t)) = (signers, threshold) {
            governance::init_governance(
                &e,
                s.clone(),
                t,
                proposal_ttl.unwrap_or(17280 * 7), // default 7 days
                guardian,
            )?;
            storage::set_multisig(&e);
            events::governance_migrated(&e, admin.clone(), s.len(), t);
        }

        // Bootstrap version tracking.
        if let Some(hash) = initial_wasm_hash {
            upgrade::set_initial_version(&e, hash);
        }

        events::initialized(&e, admin, fee_rate);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Switch a single-admin contract to multi-sig governance (one-way).
    pub fn migrate_to_multisig(
        e: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        proposal_ttl: u64,
        guardian: Option<Address>,
    ) -> Result<(), ContractError> {
        governance::migrate_to_multisig(&e, admin, signers, threshold, proposal_ttl, guardian)
    }

    // ── Single-admin operations (rejected in multi-sig mode) ──────────────────

    pub fn set_admin(e: Env, new_admin: Address) -> Result<(), ContractError> {
        if storage::is_multisig(&e) {
            return Err(ContractError::UseGovernance);
        }
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.storage().instance().set(&StorageKey::Admin, &new_admin);
        events::admin_changed(&e, admin, new_admin);
        extend_instance_ttl(&e);
        Ok(())
    }

    pub fn register_pool(e: Env, pool: Address) -> Result<(), ContractError> {
        if storage::is_multisig(&e) {
            return Err(ContractError::UseGovernance);
        }
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
        if storage::is_multisig(&e) {
            return Err(ContractError::UseGovernance);
        }
        storage::get_admin(&e).require_auth();
        e.storage().instance().set(&StorageKey::Paused, &true);
        events::paused(&e);
        Ok(())
    }

    pub fn unpause(e: Env) -> Result<(), ContractError> {
        if storage::is_multisig(&e) {
            return Err(ContractError::UseGovernance);
        }
        storage::get_admin(&e).require_auth();
        e.storage().instance().set(&StorageKey::Paused, &false);
        events::unpaused(&e);
        Ok(())
    }

    // ── Multi-sig governance entrypoints ──────────────────────────────────────

    /// Create a governance proposal. Returns the proposal ID.
    pub fn propose(e: Env, signer: Address, action: ProposalAction) -> Result<u64, ContractError> {
        if !storage::is_multisig(&e) {
            return Err(ContractError::NotMultiSig);
        }
        governance::propose(&e, signer, action)
    }

    /// Approve a proposal. Auto-executes when threshold is reached.
    pub fn approve_proposal(
        e: Env,
        signer: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        if !storage::is_multisig(&e) {
            return Err(ContractError::NotMultiSig);
        }
        governance::approve(&e, signer, proposal_id)
    }

    /// Manually execute a proposal once threshold has been met.
    pub fn execute_proposal(e: Env, proposal_id: u64) -> Result<(), ContractError> {
        if !storage::is_multisig(&e) {
            return Err(ContractError::NotMultiSig);
        }
        governance::execute_proposal(&e, proposal_id)
    }

    /// Cancel a proposal (proposer or any signer).
    pub fn cancel_proposal(
        e: Env,
        signer: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        if !storage::is_multisig(&e) {
            return Err(ContractError::NotMultiSig);
        }
        governance::cancel(&e, signer, proposal_id)
    }

    /// Emergency pause callable by the guardian only (no multi-sig delay).
    pub fn guardian_pause(e: Env, guardian: Address) -> Result<(), ContractError> {
        governance::guardian_pause(&e, guardian)
    }

    /// Read-only: return the governance config.
    pub fn get_governance_config(e: Env) -> Result<GovernanceConfig, ContractError> {
        governance::get_governance_config(&e)
    }

    /// Read-only: return a proposal by ID.
    pub fn get_proposal(e: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        governance::get_proposal(&e, proposal_id)
    }

    // ── Upgrade entrypoints ───────────────────────────────────────────────────

    /// Propose a time-locked upgrade (single-admin mode only).
    pub fn propose_upgrade(
        e: Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
        execute_after: u64,
    ) -> Result<(), ContractError> {
        upgrade::propose_upgrade(&e, admin, new_wasm_hash, execute_after)
    }

    /// Execute a pending upgrade after the time-lock has elapsed.
    pub fn execute_upgrade(e: Env) -> Result<(), ContractError> {
        upgrade::execute_upgrade(&e)
    }

    /// Cancel a pending upgrade (proposer only).
    pub fn cancel_upgrade(e: Env, admin: Address) -> Result<(), ContractError> {
        upgrade::cancel_upgrade(&e, admin)
    }

    /// Return the current contract version.
    pub fn get_version(e: Env) -> ContractVersion {
        upgrade::get_version_for_query(&e)
    }

    // ── Token allowlist entrypoints ─────────────────────────────────────────────

    /// Add a single token to the allowlist (single-admin mode).
    pub fn add_token(e: Env, caller: Address, info: TokenInfo) -> Result<(), ContractError> {
        tokens::add_token(&e, caller, info)
    }

    /// Remove a token from the allowlist (single-admin mode).
    pub fn remove_token(
        e: Env,
        caller: Address,
        asset: crate::types::Asset,
    ) -> Result<(), ContractError> {
        tokens::remove_token(&e, caller, asset)
    }

    /// Update token metadata without re-adding (single-admin mode).
    pub fn update_token(
        e: Env,
        caller: Address,
        asset: crate::types::Asset,
        updated: TokenInfo,
    ) -> Result<(), ContractError> {
        tokens::update_token(&e, caller, asset, updated)
    }

    /// Batch-add up to 10 tokens in a single call (single-admin mode).
    pub fn add_tokens_batch(
        e: Env,
        caller: Address,
        token_list: Vec<TokenInfo>,
    ) -> Result<(), ContractError> {
        tokens::add_tokens_batch(&e, caller, token_list)
    }

    /// Read-only: return `true` if the asset is on the allowlist.
    pub fn is_token_allowed(e: Env, asset: crate::types::Asset) -> bool {
        tokens::is_token_allowed(&e, &asset)
    }

    /// Read-only: return token metadata.
    pub fn get_token_info(
        e: Env,
        asset: crate::types::Asset,
    ) -> Option<TokenInfo> {
        tokens::get_token_info(&e, &asset)
    }

    /// Read-only: total count of active allowlisted tokens.
    pub fn get_token_count(e: Env) -> u32 {
        tokens::get_token_count(&e)
    }

    /// Read-only: all active assets in a given category.
    pub fn get_tokens_by_category(
        e: Env,
        category: TokenCategory,
    ) -> Vec<crate::types::Asset> {
        tokens::get_tokens_by_category(&e, category)
    }

    // ── Read-only getters ─────────────────────────────────────────────────────

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

    /// Public entry point for users to get quotes
    pub fn get_quote(e: Env, amount_in: i128, route: Route) -> Result<QuoteResult, ContractError> {
        if amount_in <= 0 || route.hops.is_empty() || route.hops.len() > 4 {
            return Err(ContractError::InvalidRoute);
        }
        // Validate every asset in the route is on the allowlist.
        tokens::validate_route_assets(&e, &route)?;

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
        // Validate every asset in the route is on the allowlist.
        tokens::validate_route_assets(&e, &params.route)?;

        if e.ledger().sequence() as u64 > params.deadline {
            return Err(ContractError::DeadlineExceeded);
        }

        if params.route.hops.is_empty() || params.route.hops.len() > 4 {
            return Err(ContractError::InvalidRoute);
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

            if !is_supported_pool(&e, hop.pool.clone()) {
                return Err(ContractError::PoolNotSupported);
            }

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

        let fee_rate = get_fee_rate(&e);
        let fee_amount = (current_input_amount * fee_rate as i128) / 10000;
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
            &get_fee_to(&e),
            fee_amount,
        );

        increment_nonce(&e, sender.clone());

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
