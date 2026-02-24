use crate::types::Route;
use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

pub fn initialized(e: &Env, admin: Address, fee_rate: u32) {
    let topics = (Symbol::new(e, "StellarRoute"), symbol_short!("init"));
    e.events().publish(topics, (admin, fee_rate));
}

pub fn admin_changed(e: &Env, old_admin: Address, new_admin: Address) {
    let topics = (Symbol::new(e, "StellarRoute"), symbol_short!("adm_chg"));
    e.events().publish(topics, (old_admin, new_admin));
}

pub fn pool_registered(e: &Env, pool_address: Address) {
    let topics = (Symbol::new(e, "StellarRoute"), symbol_short!("reg_pol"));
    e.events().publish(topics, pool_address);
}

pub fn paused(e: &Env) {
    let topics = (Symbol::new(e, "StellarRoute"), symbol_short!("paused"));
    e.events().publish(topics, ());
}

pub fn unpaused(e: &Env) {
    let topics = (Symbol::new(e, "StellarRoute"), symbol_short!("unpaused"));
    e.events().publish(topics, ());
}

pub fn swap_executed(
    e: &Env,
    sender: Address,
    amount_in: i128,
    amount_out: i128,
    fee: i128,
    route: Route,
) {
    let topics = (
        Symbol::new(e, "StellarRoute"),
        symbol_short!("swap"),
        sender,
    );
    e.events().publish(
        topics,
        (amount_in, amount_out, fee, route, e.ledger().sequence()),
    );
}

// --- MEV Protection Events ---

pub fn high_impact_swap(e: &Env, sender: Address, impact_bps: u32, amount_in: i128) {
    let topics = (
        Symbol::new(e, "StellarRoute"),
        symbol_short!("hi_imp"),
        sender,
    );
    e.events().publish(topics, (impact_bps, amount_in));
}

pub fn rate_limit_hit(e: &Env, sender: Address, swap_count: u32, window: u32) {
    let topics = (
        Symbol::new(e, "StellarRoute"),
        symbol_short!("rl_hit"),
        sender,
    );
    e.events().publish(topics, (swap_count, window));
}

pub fn commitment_created(
    e: &Env,
    sender: Address,
    commitment_hash: BytesN<32>,
    deposit_amount: i128,
) {
    let topics = (
        Symbol::new(e, "StellarRoute"),
        symbol_short!("cmt_new"),
        sender,
    );
    e.events()
        .publish(topics, (commitment_hash, deposit_amount));
}

pub fn commitment_revealed(e: &Env, sender: Address, commitment_hash: BytesN<32>) {
    let topics = (
        Symbol::new(e, "StellarRoute"),
        symbol_short!("cmt_rev"),
        sender,
    );
    e.events().publish(topics, commitment_hash);
}

