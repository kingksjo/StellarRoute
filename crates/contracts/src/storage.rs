use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum StorageKey {
    Admin,
    FeeRate,
    Paused,
    SupportedPool(Address),
}

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&StorageKey::Admin).unwrap()
}

pub fn set_admin(e: &Env, address: &Address) {
    e.storage().instance().set(&StorageKey::Admin, address);
}

pub fn is_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&StorageKey::Paused)
        .unwrap_or(false)
}

pub fn set_pause(e: &Env, paused: bool) {
    e.storage().instance().set(&StorageKey::Paused, &paused);
}

pub fn is_supported_pool(e: &Env, pool: Address) -> bool {
    e.storage()
        .persistent()
        .has(&StorageKey::SupportedPool(pool))
}

// TTL Extension Helper (Crucial for Persistent storage)
pub fn extend_instance_ttl(e: &Env) {
    e.storage().instance().extend_ttl(100_000, 500_000);
}
