use crate::types::{
    Asset, CommitmentData, ContractVersion, GovernanceConfig, MevConfig, PendingUpgrade, Proposal,
    TokenCategory, TokenInfo,
};
use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
pub enum StorageKey {
    Admin,
    FeeRate,
    FeeTo,
    Paused,
    SupportedPool(Address),
    PoolCount,
    SwapNonce(Address),
    // ── Multi-sig governance ─────────────────────────────────────────────────
    /// Stored: GovernanceConfig
    Governance,
    /// True once migrate_to_multisig has been called (irreversible).
    IsMultiSig,
    /// Guardian address for emergency pause only.
    Guardian,
    /// Monotonically-increasing proposal ID counter.
    ProposalCounter,
    /// Stored: Proposal  (Persistent, keyed by proposal ID)
    ProposalEntry(u64),
    // ── Upgrade ──────────────────────────────────────────────────────────────
    /// Current deployed version. Stored: ContractVersion  (Instance)
    ContractVersionKey,
    /// Audit trail snapshot at ledger sequence N. Stored: ContractVersion (Persistent)
    VersionHistory(u64),
    /// Pending time-locked upgrade. Stored: PendingUpgrade  (Instance)
    PendingUpgradeKey,
    /// Tracks whether post-upgrade migration has run for a given (major,minor,patch) triplet.
    MigrationDone(u32, u32, u32),
    // ── Token allowlist ──────────────────────────────────────────────────────
    /// Stored: TokenInfo  (Persistent, keyed by Asset)
    AllowedToken(Asset),
    /// Total count of allowlisted tokens.  (Instance)
    TokenCount,
    // MEV protection keys
    MevConfig,
    Commitment(BytesN<32>),
    AccountSwapCount(Address),
    AccountSwapWindowStart(Address),
    Whitelisted(Address),
    LatestKnownPrice(Address, Address),
}

const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS;
const TEMP_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS;
const TEMP_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS / 2;

pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// --- Core storage helpers ---

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&StorageKey::Admin).unwrap()
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&StorageKey::Admin, admin);
}

pub fn get_fee_rate(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&StorageKey::FeeRate)
        .unwrap_or(0)
}

pub fn set_fee_rate(e: &Env, rate: u32) {
    e.storage().instance().set(&StorageKey::FeeRate, &rate);
}

pub fn get_fee_to(e: &Env) -> Address {
    e.storage().instance().get(&StorageKey::FeeTo).unwrap()
}

pub fn get_fee_to_optional(e: &Env) -> Option<Address> {
    e.storage().instance().get(&StorageKey::FeeTo)
}

pub fn get_pool_count(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&StorageKey::PoolCount)
        .unwrap_or(0)
}

pub fn set_pool_count(e: &Env, count: u32) {
    e.storage().instance().set(&StorageKey::PoolCount, &count);
}

pub fn get_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&StorageKey::Paused)
        .unwrap_or(false)
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&StorageKey::Admin)
}

pub fn is_supported_pool(e: &Env, pool: Address) -> bool {
    e.storage()
        .persistent()
        .has(&StorageKey::SupportedPool(pool))
}

pub fn get_nonce(e: &Env, address: Address) -> i128 {
    let key = StorageKey::SwapNonce(address);
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub fn increment_nonce(e: &Env, address: Address) {
    let key = StorageKey::SwapNonce(address.clone());
    let current = get_nonce(e, address);
    e.storage().persistent().set(&key, &(current + 1));
}

pub fn transfer_asset(e: &Env, asset: &Asset, from: &Address, to: &Address, amount: i128) {
    if let Asset::Soroban(address) = asset {
        let client = soroban_sdk::token::Client::new(e, address);
        client.transfer(from, to, &amount);
    }
}

// ─── Multi-sig governance helpers ────────────────────────────────────────────

pub fn is_multisig(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&StorageKey::IsMultiSig)
        .unwrap_or(false)
}

pub fn set_multisig(e: &Env) {
    e.storage().instance().set(&StorageKey::IsMultiSig, &true);
}

pub fn get_governance(e: &Env) -> GovernanceConfig {
    e.storage().instance().get(&StorageKey::Governance).unwrap()
}

pub fn set_governance(e: &Env, config: &GovernanceConfig) {
    e.storage().instance().set(&StorageKey::Governance, config);
}

pub fn has_guardian(e: &Env) -> bool {
    e.storage().instance().has(&StorageKey::Guardian)
}

pub fn get_guardian(e: &Env) -> Option<Address> {
    e.storage().instance().get(&StorageKey::Guardian)
}

pub fn set_guardian(e: &Env, guardian: &Address) {
    e.storage().instance().set(&StorageKey::Guardian, guardian);
}

pub fn next_proposal_id(e: &Env) -> u64 {
    let id: u64 = e
        .storage()
        .instance()
        .get(&StorageKey::ProposalCounter)
        .unwrap_or(0);
    let next = id + 1;
    e.storage()
        .instance()
        .set(&StorageKey::ProposalCounter, &next);
    next
}

pub fn get_proposal(e: &Env, id: u64) -> Option<Proposal> {
    e.storage().persistent().get(&StorageKey::ProposalEntry(id))
}

pub fn save_proposal(e: &Env, proposal: &Proposal) {
    let key = StorageKey::ProposalEntry(proposal.id);
    e.storage().persistent().set(&key, proposal);
    e.storage().persistent().extend_ttl(&key, 17280, 17280 * 30);
}

// ─── Upgrade helpers ─────────────────────────────────────────────────────────

pub fn get_contract_version(e: &Env) -> Option<ContractVersion> {
    e.storage().instance().get(&StorageKey::ContractVersionKey)
}

pub fn set_contract_version(e: &Env, version: &ContractVersion) {
    e.storage()
        .instance()
        .set(&StorageKey::ContractVersionKey, version);
    // Also snapshot in persistent storage for audit trail
    let key = StorageKey::VersionHistory(version.upgraded_at);
    e.storage().persistent().set(&key, version);
    e.storage()
        .persistent()
        .extend_ttl(&key, 17280, 17280 * 365);
}

pub fn get_pending_upgrade(e: &Env) -> Option<PendingUpgrade> {
    e.storage().instance().get(&StorageKey::PendingUpgradeKey)
}

pub fn set_pending_upgrade(e: &Env, pending: &PendingUpgrade) {
    e.storage()
        .instance()
        .set(&StorageKey::PendingUpgradeKey, pending);
}

pub fn clear_pending_upgrade(e: &Env) {
    e.storage()
        .instance()
        .remove(&StorageKey::PendingUpgradeKey);
}

pub fn is_migration_done(e: &Env, major: u32, minor: u32, patch: u32) -> bool {
    e.storage()
        .persistent()
        .has(&StorageKey::MigrationDone(major, minor, patch))
}

pub fn set_migration_done(e: &Env, major: u32, minor: u32, patch: u32) {
    let key = StorageKey::MigrationDone(major, minor, patch);
    e.storage().persistent().set(&key, &true);
    e.storage()
        .persistent()
        .extend_ttl(&key, 17280, 17280 * 365);
}

// ─── Token allowlist helpers ──────────────────────────────────────────────────

pub fn get_token_info(e: &Env, asset: &Asset) -> Option<TokenInfo> {
    e.storage()
        .persistent()
        .get(&StorageKey::AllowedToken(asset.clone()))
}

pub fn is_token_allowed(e: &Env, asset: &Asset) -> bool {
    e.storage()
        .persistent()
        .has(&StorageKey::AllowedToken(asset.clone()))
}

pub fn save_token_info(e: &Env, info: &TokenInfo) {
    let key = StorageKey::AllowedToken(info.asset.clone());
    e.storage().persistent().set(&key, info);
    e.storage()
        .persistent()
        .extend_ttl(&key, 17280, 17280 * 365);
}

pub fn remove_token(e: &Env, asset: &Asset) {
    e.storage()
        .persistent()
        .remove(&StorageKey::AllowedToken(asset.clone()));
}

pub fn get_token_count(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&StorageKey::TokenCount)
        .unwrap_or(0)
}

pub fn set_token_count(e: &Env, count: u32) {
    e.storage().instance().set(&StorageKey::TokenCount, &count);
}

/// Return all tokens whose category matches `filter`.
/// Iterates the provided list of all known assets (callers build this
/// via repeated get_token_info calls or by storing a separate index —
/// here we expose the building block; the tokens module keeps a by-category
/// index).
pub fn get_tokens_by_category_key(e: &Env, asset: &Asset) -> Option<TokenCategory> {
    get_token_info(e, asset).map(|i| i.category)
}
// --- MEV Config ---

pub fn get_mev_config(e: &Env) -> Option<MevConfig> {
    e.storage().instance().get(&StorageKey::MevConfig)
}

pub fn set_mev_config(e: &Env, config: &MevConfig) {
    e.storage().instance().set(&StorageKey::MevConfig, config);
}

// --- Commitment storage (Temporary) ---

pub fn get_commitment(e: &Env, hash: &BytesN<32>) -> Option<CommitmentData> {
    let key = StorageKey::Commitment(hash.clone());
    e.storage().temporary().get(&key)
}

pub fn set_commitment(e: &Env, hash: &BytesN<32>, data: &CommitmentData, ttl_ledgers: u32) {
    let key = StorageKey::Commitment(hash.clone());
    e.storage().temporary().set(&key, data);
    e.storage()
        .temporary()
        .extend_ttl(&key, ttl_ledgers, ttl_ledgers);
}

pub fn remove_commitment(e: &Env, hash: &BytesN<32>) {
    let key = StorageKey::Commitment(hash.clone());
    e.storage().temporary().remove(&key);
}

// --- Rate limiting (Temporary) ---

pub fn get_account_swap_count(e: &Env, address: &Address) -> u32 {
    let key = StorageKey::AccountSwapCount(address.clone());
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn set_account_swap_count(e: &Env, address: &Address, count: u32, ttl_ledgers: u32) {
    let key = StorageKey::AccountSwapCount(address.clone());
    e.storage().temporary().set(&key, &count);
    e.storage()
        .temporary()
        .extend_ttl(&key, ttl_ledgers, ttl_ledgers);
}

pub fn get_account_swap_window_start(e: &Env, address: &Address) -> u32 {
    let key = StorageKey::AccountSwapWindowStart(address.clone());
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn set_account_swap_window_start(e: &Env, address: &Address, start: u32, ttl_ledgers: u32) {
    let key = StorageKey::AccountSwapWindowStart(address.clone());
    e.storage().temporary().set(&key, &start);
    e.storage()
        .temporary()
        .extend_ttl(&key, ttl_ledgers, ttl_ledgers);
}

// --- Whitelist (Persistent) ---

pub fn is_whitelisted(e: &Env, address: &Address) -> bool {
    let key = StorageKey::Whitelisted(address.clone());
    e.storage().persistent().get(&key).unwrap_or(false)
}

pub fn set_whitelisted(e: &Env, address: &Address, whitelisted: bool) {
    let key = StorageKey::Whitelisted(address.clone());
    e.storage().persistent().set(&key, &whitelisted);
    if whitelisted {
        e.storage()
            .persistent()
            .extend_ttl(&key, DAY_IN_LEDGERS, DAY_IN_LEDGERS * 30);
    }
}

// --- Latest known price (Instance) ---

pub fn get_latest_known_price(e: &Env, token_a: &Address, token_b: &Address) -> Option<i128> {
    let key = StorageKey::LatestKnownPrice(token_a.clone(), token_b.clone());
    e.storage().instance().get(&key)
}

pub fn set_latest_known_price(e: &Env, token_a: &Address, token_b: &Address, price: i128) {
    let key = StorageKey::LatestKnownPrice(token_a.clone(), token_b.clone());
    e.storage().instance().set(&key, &price);
}
