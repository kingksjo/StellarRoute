use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::router::{StellarRoute, StellarRouteClient};

fn setup() -> (Env, StellarRouteClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let fee_to = Address::generate(&env);
    (env, client, admin, fee_to)
}

// ── version ──────────────────────────────────────────────────────────

#[test]
fn test_version_returns_constant() {
    let (_env, client, _admin, _fee_to) = setup();
    assert_eq!(client.version(), 1);
}

// ── get_admin ────────────────────────────────────────────────────────

#[test]
fn test_get_admin_uninitialized() {
    let (_env, client, _admin, _fee_to) = setup();
    let result = client.try_get_admin();
    assert!(result.is_err());
}

#[test]
fn test_get_admin_after_init() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_get_admin_after_set_admin() {
    let (env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
}

// ── get_fee_rate_value ───────────────────────────────────────────────

#[test]
fn test_get_fee_rate_uninitialized() {
    let (_env, client, _admin, _fee_to) = setup();
    assert_eq!(client.get_fee_rate_value(), 0);
}

#[test]
fn test_get_fee_rate_after_init() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &250, &fee_to);
    assert_eq!(client.get_fee_rate_value(), 250);
}

// ── get_fee_to_address ───────────────────────────────────────────────

#[test]
fn test_get_fee_to_address_uninitialized() {
    let (_env, client, _admin, _fee_to) = setup();
    let result = client.try_get_fee_to_address();
    assert!(result.is_err());
}

#[test]
fn test_get_fee_to_address_after_init() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    assert_eq!(client.get_fee_to_address(), fee_to);
}

// ── is_paused ────────────────────────────────────────────────────────

#[test]
fn test_is_paused_uninitialized() {
    let (_env, client, _admin, _fee_to) = setup();
    assert!(!client.is_paused());
}

#[test]
fn test_is_paused_default_false() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    assert!(!client.is_paused());
}

#[test]
fn test_is_paused_after_pause() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    client.pause();
    assert!(client.is_paused());
}

#[test]
fn test_is_paused_after_unpause() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    client.pause();
    client.unpause();
    assert!(!client.is_paused());
}

// ── get_pool_count ───────────────────────────────────────────────────

#[test]
fn test_get_pool_count_uninitialized() {
    let (_env, client, _admin, _fee_to) = setup();
    assert_eq!(client.get_pool_count(), 0);
}

#[test]
fn test_get_pool_count_after_init() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    assert_eq!(client.get_pool_count(), 0);
}

#[test]
fn test_get_pool_count_increments() {
    let (env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);

    let pool1 = Address::generate(&env);
    let pool2 = Address::generate(&env);
    client.register_pool(&pool1);
    assert_eq!(client.get_pool_count(), 1);
    client.register_pool(&pool2);
    assert_eq!(client.get_pool_count(), 2);
}

// ── is_pool_registered ──────────────────────────────────────────────

#[test]
fn test_is_pool_registered_unknown() {
    let (env, client, _admin, _fee_to) = setup();
    let pool = Address::generate(&env);
    assert!(!client.is_pool_registered(&pool));
}

#[test]
fn test_is_pool_registered_after_register() {
    let (env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    let pool = Address::generate(&env);
    client.register_pool(&pool);
    assert!(client.is_pool_registered(&pool));
}

#[test]
fn test_is_pool_registered_different_pool() {
    let (env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    let pool1 = Address::generate(&env);
    let pool2 = Address::generate(&env);
    client.register_pool(&pool1);
    assert!(client.is_pool_registered(&pool1));
    assert!(!client.is_pool_registered(&pool2));
}

// ── initialize edge cases ────────────────────────────────────────────

#[test]
fn test_initialize_twice_fails() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &100, &fee_to);
    let result = client.try_initialize(&admin, &100, &fee_to);
    assert!(result.is_err());
}

#[test]
fn test_initialize_fee_rate_too_high() {
    let (_env, client, admin, fee_to) = setup();
    let result = client.try_initialize(&admin, &1001, &fee_to);
    assert!(result.is_err());
}

#[test]
fn test_initialize_zero_fee_rate() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &0, &fee_to);
    assert_eq!(client.get_fee_rate_value(), 0);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_initialize_max_fee_rate() {
    let (_env, client, admin, fee_to) = setup();
    client.initialize(&admin, &1000, &fee_to);
    assert_eq!(client.get_fee_rate_value(), 1000);
}
