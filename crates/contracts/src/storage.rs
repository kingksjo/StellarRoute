use crate::types::{Asset, CommitmentData, MevConfig};
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
    // MEV protection keys
    MevConfig,
    Commitment(BytesN<32>),
    AccountSwapCount(Address),
    AccountSwapWindowStart(Address),
    Whitelisted(Address),
    LatestKnownPrice(Address, Address),
}

#[contracttype]
#[derive(Clone)]
pub struct InstanceConfig {
    pub admin: Address,
    pub fee_rate: u32,
    pub fee_to: Address,
    pub paused: bool,
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

// Optimized: Use Symbol for cheaper storage keys
pub fn transfer_asset(e: &Env, asset: &Asset, from: &Address, to: &Address, amount: i128) {
    if let Asset::Soroban(address) = asset {
        let client = soroban_sdk::token::Client::new(e, address);
        client.transfer(from, to, &amount);
    }
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
        .extend_ttl(&key, TEMP_LIFETIME_THRESHOLD, ttl_ledgers);
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
        .extend_ttl(&key, TEMP_LIFETIME_THRESHOLD, ttl_ledgers);
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
        .extend_ttl(&key, TEMP_LIFETIME_THRESHOLD, ttl_ledgers);
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

