use crate::types::Asset;
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum StorageKey {
    Admin,
    FeeRate,
    FeeTo,
    Paused,
    SupportedPool(Address),
    PoolCount,
    SwapNonce(Address),
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

pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// Optimized: Batch read all instance config in one operation
pub fn get_instance_config(e: &Env) -> InstanceConfig {
    let storage = e.storage().instance();
    InstanceConfig {
        admin: storage.get(&StorageKey::Admin).unwrap(),
        fee_rate: storage.get(&StorageKey::FeeRate).unwrap_or(0),
        fee_to: storage.get(&StorageKey::FeeTo).unwrap(),
        paused: storage.get(&StorageKey::Paused).unwrap_or(false),
    }
}

// Cache pool lookups during route execution
pub fn batch_check_pools(e: &Env, pools: &soroban_sdk::Vec<Address>) -> bool {
    for i in 0..pools.len() {
        let pool = pools.get(i).unwrap();
        if !e.storage().persistent().has(&StorageKey::SupportedPool(pool)) {
            return false;
        }
    }
    true
}

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

// Inline constant product calculation to avoid CCI for known pool types
#[inline(always)]
pub fn calculate_constant_product_output(
    reserve_in: i128,
    reserve_out: i128,
    amount_in: i128,
) -> i128 {
    let amount_in_with_fee = amount_in * 997;
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = (reserve_in * 1000) + amount_in_with_fee;
    numerator / denominator
}
