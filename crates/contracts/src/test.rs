//! Comprehensive test suite for the StellarRoute router contract.
//!
//! Covers: initialization, admin, pool registration, pause/unpause, quote,
//! swap execution (single/multi-hop), slippage, deadlines, error paths,
//! property checks, and end-to-end lifecycle tests.
//!
//! Run with:
//!   cargo test -p stellarroute-contracts

#![allow(dead_code)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Bytes, BytesN, Env, Vec,
};

use super::{
    errors::ContractError,
    router::{StellarRoute, StellarRouteClient},
    types::{Asset, MevConfig, PoolType, Route, RouteHop, SwapParams},
};

// ── Mock Contracts ────────────────────────────────────────────────────────────
// Each mock lives in its own submodule because `#[contractimpl]` generates
// module-level symbols (e.g. `__swap`, `__adapter_quote`) that collide when
// two contracts in the same module share method names.

mod mock_amm {
    use super::super::types::Asset;
    use soroban_sdk::{contract, contractimpl, Env};

    /// A simple AMM mock that returns 99 % of amount_in for both quotes and swaps.
    /// Accepts Asset parameters matching what the router sends via CCI.
    #[contract]
    pub struct MockAmmPool;

    #[contractimpl]
    impl MockAmmPool {
        /// Called by router via Symbol::new("adapter_quote").
        pub fn adapter_quote(
            _e: Env,
            _in_asset: Asset,
            _out_asset: Asset,
            amount_in: i128,
        ) -> i128 {
            amount_in * 99 / 100
        }

        /// Called by router via symbol_short!("swap").
        pub fn swap(
            _e: Env,
            _in_asset: Asset,
            _out_asset: Asset,
            amount_in: i128,
            min_out: i128,
        ) -> i128 {
            let out = amount_in * 99 / 100;
            if out < min_out {
                panic!("mock pool: slippage");
            }
            out
        }

        pub fn get_rsrvs(_e: Env) -> (i128, i128) {
            (1_000_000_000, 1_000_000_000)
        }
    }
}

mod mock_failing {
    use super::super::types::Asset;
    use soroban_sdk::{contract, contractimpl, Env};

    /// A pool that always panics — used to test PoolCallFailed error paths.
    #[contract]
    pub struct MockFailingPool;

    #[contractimpl]
    impl MockFailingPool {
        pub fn adapter_quote(_e: Env, _in: Asset, _out: Asset, _amount: i128) -> i128 {
            panic!("mock: pool unavailable")
        }

        pub fn swap(_e: Env, _in: Asset, _out: Asset, _amount: i128, _min: i128) -> i128 {
            panic!("mock: pool unavailable")
        }

        pub fn get_rsrvs(_e: Env) -> (i128, i128) {
            panic!("mock: pool unavailable")
        }
    }
}

use mock_amm::MockAmmPool;
use mock_failing::MockFailingPool;

// ── Test Utilities ────────────────────────────────────────────────────────────

/// Create a fresh Env with all auth mocked — standard for unit tests.
fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Deploy and initialise the router. Returns (admin, fee_to, client).
fn deploy_router(env: &Env) -> (Address, Address, StellarRouteClient<'_>) {
    let admin = Address::generate(env);
    let fee_to = Address::generate(env);
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(env, &id);
    client.initialize(&admin, &30_u32, &fee_to); // 0.3 % protocol fee
    (admin, fee_to, client)
}

fn deploy_mock_pool(env: &Env) -> Address {
    env.register_contract(None, MockAmmPool)
}

fn deploy_failing_pool(env: &Env) -> Address {
    env.register_contract(None, MockFailingPool)
}

fn make_route(env: &Env, pool: &Address, hops: u32) -> Route {
    let mut v = Vec::new(env);
    for _ in 0..hops {
        v.push_back(RouteHop {
            source: Asset::Native,
            destination: Asset::Native,
            pool: pool.clone(),
            pool_type: PoolType::AmmConstProd,
        });
    }
    Route {
        hops: v,
        estimated_output: 990,
        min_output: 900,
        expires_at: 99_999,
    }
}

fn current_seq(env: &Env) -> u64 {
    env.ledger().sequence() as u64
}

fn swap_params_for(
    env: &Env,
    route: Route,
    amount_in: i128,
    min_out: i128,
    deadline: u64,
) -> SwapParams {
    SwapParams {
        route,
        amount_in,
        min_amount_out: min_out,
        recipient: Address::generate(env),
        deadline,
        not_before: 0,
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    }
}

fn simple_swap(
    env: &Env,
    client: &StellarRouteClient<'_>,
    pool: &Address,
) -> crate::types::SwapResult {
    let sender = Address::generate(env);
    let route = make_route(env, pool, 1);
    let params = swap_params_for(env, route, 1000, 0, current_seq(env) + 100);
    client.execute_swap(&sender, &params)
}

// ── Initialization Tests ──────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let env = setup_env();
    deploy_router(&env);
}

#[test]
fn test_initialize_double_returns_error() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let result = client.try_initialize(&Address::generate(&env), &30_u32, &Address::generate(&env));
    assert_eq!(result, Err(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn test_initialize_max_valid_fee() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    // 1000 bps (10 %) is the maximum allowed value
    client.initialize(
        &Address::generate(&env),
        &1000_u32,
        &Address::generate(&env),
    );
}

#[test]
fn test_initialize_invalid_fee() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    let result = client.try_initialize(
        &Address::generate(&env),
        &1001_u32,
        &Address::generate(&env),
    );
    assert_eq!(result, Err(Ok(ContractError::InvalidAmount)));
}

#[test]
fn test_initialize_zero_fee() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    client.initialize(&Address::generate(&env), &0_u32, &Address::generate(&env));
}

// ── Admin Tests ───────────────────────────────────────────────────────────────

#[test]
fn test_set_admin_success() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.set_admin(&Address::generate(&env));
}

#[test]
fn test_set_admin_emits_event() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let events_before = env.events().all().len();
    client.set_admin(&Address::generate(&env));
    assert!(env.events().all().len() > events_before);
}

#[test]
fn test_set_admin_then_pool_ops_still_work() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.set_admin(&Address::generate(&env));
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool); // must still succeed
}

// ── Pool Registration Tests ───────────────────────────────────────────────────

#[test]
fn test_register_pool_success() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.register_pool(&deploy_mock_pool(&env));
}

#[test]
fn test_register_pool_duplicate() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = client.try_register_pool(&pool);
    assert_eq!(result, Err(Ok(ContractError::PoolNotSupported)));
}

#[test]
fn test_register_multiple_distinct_pools() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.register_pool(&deploy_mock_pool(&env));
    client.register_pool(&deploy_mock_pool(&env));
    client.register_pool(&deploy_mock_pool(&env));
}

// ── Pause / Unpause Tests ─────────────────────────────────────────────────────

#[test]
fn test_pause_blocks_swaps() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.pause();

    let result = client.try_execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 1),
            1000,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert_eq!(result, Err(Ok(ContractError::Paused)));
}

#[test]
fn test_pause_does_not_block_registration() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.pause();
    client.register_pool(&deploy_mock_pool(&env));
}

#[test]
fn test_unpause_resumes_swaps() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.pause();
    client.unpause();

    let result = client.try_execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 1),
            1000,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert!(result.is_ok());
}

#[test]
fn test_pause_unpause_toggle() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    client.pause();
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                1000,
                0,
                current_seq(&env) + 100
            ),
        ),
        Err(Ok(ContractError::Paused))
    );

    client.unpause();
    assert!(client
        .try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                1000,
                0,
                current_seq(&env) + 100
            ),
        )
        .is_ok());
}

// ── Get Quote Tests ───────────────────────────────────────────────────────────

#[test]
fn test_get_quote_single_hop() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    let quote = client.get_quote(&1000, &make_route(&env, &pool, 1));
    // pool returns 99 % (990), protocol fee 30 bps (2), output = 988
    assert_eq!(quote.expected_output, 988);
    assert_eq!(quote.fee_amount, 2);
}

#[test]
fn test_get_quote_negative_amount_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    assert_eq!(
        client.try_get_quote(&-1, &make_route(&env, &pool, 1)),
        Err(Ok(ContractError::InvalidRoute))
    );
}

#[test]
fn test_get_quote_zero_amount_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    assert_eq!(
        client.try_get_quote(&0, &make_route(&env, &pool, 1)),
        Err(Ok(ContractError::InvalidRoute))
    );
}

#[test]
fn test_get_quote_empty_hops_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let empty = Route {
        hops: Vec::new(&env),
        estimated_output: 0,
        min_output: 0,
        expires_at: 99_999,
    };
    assert_eq!(
        client.try_get_quote(&1000, &empty),
        Err(Ok(ContractError::InvalidRoute))
    );
}

#[test]
fn test_get_quote_too_many_hops_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    assert_eq!(
        client.try_get_quote(&1000, &make_route(&env, &pool, 5)),
        Err(Ok(ContractError::InvalidRoute))
    );
}

#[test]
fn test_get_quote_unregistered_pool_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env); // not registered
    assert_eq!(
        client.try_get_quote(&1000, &make_route(&env, &pool, 1)),
        Err(Ok(ContractError::PoolNotSupported))
    );
}

#[test]
fn test_get_quote_failing_pool_returns_error() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_failing_pool(&env);
    client.register_pool(&pool);
    assert_eq!(
        client.try_get_quote(&1000, &make_route(&env, &pool, 1)),
        Err(Ok(ContractError::PoolCallFailed))
    );
}

#[test]
fn test_get_quote_more_hops_more_price_impact() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let q1 = client.get_quote(&1000, &make_route(&env, &pool, 1));
    let q3 = client.get_quote(&1000, &make_route(&env, &pool, 3));
    assert!(q3.price_impact_bps > q1.price_impact_bps);
}

// ── Single-Hop Swap Tests ─────────────────────────────────────────────────────

#[test]
fn test_swap_single_hop_success() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = simple_swap(&env, &client, &pool);
    assert_eq!(result.amount_in, 1000);
    assert_eq!(result.amount_out, 988);
}

#[test]
fn test_swap_emits_event() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let events_before = env.events().all().len();
    simple_swap(&env, &client, &pool);
    assert!(env.events().all().len() > events_before);
}

#[test]
fn test_swap_result_fields() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = simple_swap(&env, &client, &pool);
    assert_eq!(result.amount_in, 1000);
    assert!(result.amount_out > 0);
    assert_eq!(result.executed_at, current_seq(&env));
}

// ── Multi-Hop Swap Tests ──────────────────────────────────────────────────────

#[test]
fn test_swap_two_hops() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 2),
            1000,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert!(result.amount_out > 0);
}

#[test]
fn test_swap_three_hops() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 3),
            10_000,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert!(result.amount_out > 0);
}

#[test]
fn test_swap_max_hops() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 4),
            10_000,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert!(result.amount_out > 0);
}

#[test]
fn test_swap_too_many_hops_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 5),
                1000,
                0,
                current_seq(&env) + 100
            ),
        ),
        Err(Ok(ContractError::InvalidRoute))
    );
}

// ── Slippage & Deadline Tests ─────────────────────────────────────────────────

#[test]
fn test_swap_slippage_exceeded() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    // pool out 990, fee → 988 net; require 999 → fail
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                1000,
                999,
                current_seq(&env) + 100
            ),
        ),
        Err(Ok(ContractError::SlippageExceeded))
    );
}

#[test]
fn test_swap_slippage_exact_minimum_succeeds() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    // min_amount_out == expected output (988)
    let result = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 1),
            1000,
            988,
            current_seq(&env) + 100,
        ),
    );
    assert_eq!(result.amount_out, 988);
}

#[test]
fn test_swap_deadline_exceeded() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    env.ledger().with_mut(|li| li.sequence_number = 1000);
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(&env, make_route(&env, &pool, 1), 1000, 0, 999),
        ),
        Err(Ok(ContractError::DeadlineExceeded))
    );
}

#[test]
fn test_swap_deadline_exact_boundary() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    env.ledger().with_mut(|li| li.sequence_number = 100);

    // deadline == sequence → NOT exceeded (check is strictly `>`)
    assert!(client
        .try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(&env, make_route(&env, &pool, 1), 1000, 0, 100),
        )
        .is_ok());

    // deadline == sequence - 1 → exceeded
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(&env, make_route(&env, &pool, 1), 1000, 0, 99),
        ),
        Err(Ok(ContractError::DeadlineExceeded))
    );
}

// ── Error Path Tests ──────────────────────────────────────────────────────────

#[test]
fn test_swap_zero_amount_produces_zero_output() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    let result = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 1),
            0,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert_eq!(result.amount_out, 0);
}

#[test]
fn test_swap_empty_route_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let empty = Route {
        hops: Vec::new(&env),
        estimated_output: 0,
        min_output: 0,
        expires_at: 99_999,
    };
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(&env, empty, 1000, 0, current_seq(&env) + 100),
        ),
        Err(Ok(ContractError::InvalidRoute))
    );
}

#[test]
fn test_swap_unregistered_pool_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env); // not registered
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                1000,
                0,
                current_seq(&env) + 100
            ),
        ),
        Err(Ok(ContractError::PoolNotSupported))
    );
}

#[test]
fn test_swap_pool_call_failure() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_failing_pool(&env);
    client.register_pool(&pool);
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                1000,
                0,
                current_seq(&env) + 100
            ),
        ),
        Err(Ok(ContractError::PoolCallFailed))
    );
}

#[test]
fn test_swap_while_paused_fails() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.pause();
    assert_eq!(
        client.try_execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                1000,
                0,
                current_seq(&env) + 100
            ),
        ),
        Err(Ok(ContractError::Paused))
    );
}

// ── Property-Based Tests ──────────────────────────────────────────────────────

#[test]
fn property_output_is_always_less_than_input() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    for amount in [100_i128, 1_000, 10_000, 100_000, 1_000_000] {
        let result = client.execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                amount,
                0,
                current_seq(&env) + 100,
            ),
        );
        assert!(
            result.amount_out < amount,
            "output {} must be < input {} (fees expected)",
            result.amount_out,
            amount
        );
        assert!(result.amount_out >= 0);
    }
}

#[test]
fn property_fee_deducted_at_correct_rate() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    // pool 99 % → protocol fee 30 bps
    for amount_in in [1_000_i128, 10_000, 100_000] {
        let result = client.execute_swap(
            &Address::generate(&env),
            &swap_params_for(
                &env,
                make_route(&env, &pool, 1),
                amount_in,
                0,
                current_seq(&env) + 100,
            ),
        );
        let pool_out = amount_in * 99 / 100;
        let fee = pool_out * 30 / 10000;
        assert_eq!(result.amount_out, pool_out - fee);
    }
}

#[test]
fn property_more_hops_means_less_output() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    let amount = 1_000_000_i128;

    let sw1 = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 1),
            amount,
            0,
            current_seq(&env) + 100,
        ),
    );
    let sw4 = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 4),
            amount,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert!(
        sw4.amount_out < sw1.amount_out,
        "4-hop {} should be < 1-hop {}",
        sw4.amount_out,
        sw1.amount_out
    );
}

#[test]
fn property_all_contract_errors_are_reachable() {
    let env = setup_env();

    // AlreadyInitialized
    let (_, _, client) = deploy_router(&env);
    assert_eq!(
        client.try_initialize(&Address::generate(&env), &30_u32, &Address::generate(&env)),
        Err(Ok(ContractError::AlreadyInitialized))
    );

    // InvalidAmount
    {
        let c = StellarRouteClient::new(&env, &env.register_contract(None, StellarRoute));
        assert_eq!(
            c.try_initialize(
                &Address::generate(&env),
                &1001_u32,
                &Address::generate(&env)
            ),
            Err(Ok(ContractError::InvalidAmount))
        );
    }

    // PoolNotSupported (duplicate registration)
    {
        let (_, _, c) = deploy_router(&env);
        let pool = deploy_mock_pool(&env);
        c.register_pool(&pool);
        assert_eq!(
            c.try_register_pool(&pool),
            Err(Ok(ContractError::PoolNotSupported))
        );
    }

    // Paused
    {
        let (_, _, c) = deploy_router(&env);
        let pool = deploy_mock_pool(&env);
        c.register_pool(&pool);
        c.pause();
        assert_eq!(
            c.try_execute_swap(
                &Address::generate(&env),
                &swap_params_for(
                    &env,
                    make_route(&env, &pool, 1),
                    1000,
                    0,
                    current_seq(&env) + 100
                ),
            ),
            Err(Ok(ContractError::Paused))
        );
    }

    // InvalidRoute (too many hops)
    {
        let (_, _, c) = deploy_router(&env);
        let pool = deploy_mock_pool(&env);
        c.register_pool(&pool);
        assert_eq!(
            c.try_execute_swap(
                &Address::generate(&env),
                &swap_params_for(
                    &env,
                    make_route(&env, &pool, 5),
                    1000,
                    0,
                    current_seq(&env) + 100
                ),
            ),
            Err(Ok(ContractError::InvalidRoute))
        );
    }

    // DeadlineExceeded
    {
        let (_, _, c) = deploy_router(&env);
        let pool = deploy_mock_pool(&env);
        c.register_pool(&pool);
        env.ledger().with_mut(|li| li.sequence_number = 500);
        assert_eq!(
            c.try_execute_swap(
                &Address::generate(&env),
                &swap_params_for(&env, make_route(&env, &pool, 1), 1000, 0, 499),
            ),
            Err(Ok(ContractError::DeadlineExceeded))
        );
        env.ledger().with_mut(|li| li.sequence_number = 0);
    }

    // PoolCallFailed
    {
        let (_, _, c) = deploy_router(&env);
        let pool = deploy_failing_pool(&env);
        c.register_pool(&pool);
        assert_eq!(
            c.try_execute_swap(
                &Address::generate(&env),
                &swap_params_for(
                    &env,
                    make_route(&env, &pool, 1),
                    1000,
                    0,
                    current_seq(&env) + 100
                ),
            ),
            Err(Ok(ContractError::PoolCallFailed))
        );
    }

    // SlippageExceeded
    {
        let (_, _, c) = deploy_router(&env);
        let pool = deploy_mock_pool(&env);
        c.register_pool(&pool);
        assert_eq!(
            c.try_execute_swap(
                &Address::generate(&env),
                &swap_params_for(
                    &env,
                    make_route(&env, &pool, 1),
                    1000,
                    999,
                    current_seq(&env) + 100
                ),
            ),
            Err(Ok(ContractError::SlippageExceeded))
        );
    }
}

// ── Integration / Lifecycle Tests ─────────────────────────────────────────────

#[test]
fn test_full_lifecycle() {
    let env = setup_env();

    // 1. Deploy & initialise
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    client.initialize(&Address::generate(&env), &30_u32, &Address::generate(&env));

    // 2. Register pool
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    // 3. Get a quote
    let quote = client.get_quote(&1000, &make_route(&env, &pool, 1));
    assert_eq!(quote.expected_output, 988);

    // 4. Execute a swap — output should match the quote
    let result = client.execute_swap(
        &Address::generate(&env),
        &swap_params_for(
            &env,
            make_route(&env, &pool, 1),
            1000,
            0,
            current_seq(&env) + 100,
        ),
    );
    assert_eq!(result.amount_out, quote.expected_output);
}

#[test]
fn test_multi_user_swaps() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    let mut total_out = 0_i128;
    for _ in 0..5 {
        let r = simple_swap(&env, &client, &pool);
        assert!(r.amount_out > 0);
        total_out += r.amount_out;
    }
    assert_eq!(total_out, 988 * 5);
}

#[test]
fn test_swap_then_admin_change_does_not_affect_pools() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    let r1 = simple_swap(&env, &client, &pool);
    assert!(r1.amount_out > 0);

    client.set_admin(&Address::generate(&env));

    let r2 = simple_swap(&env, &client, &pool);
    assert_eq!(r1.amount_out, r2.amount_out);
}

#[test]
fn test_initialize_emits_event() {
    let env = setup_env();
    deploy_router(&env);
    assert!(!env.events().all().is_empty());
}

#[test]
fn test_pause_unpause_emit_events() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let before = env.events().all().len();
    client.pause();
    client.unpause();
    assert!(env.events().all().len() > before);
}

// ── Accessor / Getter Tests (from main) ───────────────────────────────────────

#[test]
fn test_version_returns_constant() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    assert_eq!(client.version(), 2);
}

#[test]
fn test_get_admin_uninitialized() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    assert!(client.try_get_admin().is_err());
}

#[test]
fn test_get_admin_after_init() {
    let env = setup_env();
    let (admin, _, client) = deploy_router(&env);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_get_admin_after_set_admin() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_get_fee_rate_uninitialized() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    assert_eq!(client.get_fee_rate_value(), 0);
}

#[test]
fn test_get_fee_rate_after_init() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    client.initialize(&Address::generate(&env), &250_u32, &Address::generate(&env));
    assert_eq!(client.get_fee_rate_value(), 250);
}

#[test]
fn test_get_fee_to_address_uninitialized() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    assert!(client.try_get_fee_to_address().is_err());
}

#[test]
fn test_get_fee_to_address_after_init() {
    let env = setup_env();
    let fee_to = Address::generate(&env);
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    client.initialize(&Address::generate(&env), &100_u32, &fee_to);
    assert_eq!(client.get_fee_to_address(), fee_to);
}

#[test]
fn test_is_paused_uninitialized() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    assert!(!client.is_paused());
}

#[test]
fn test_is_paused_default_false() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    assert!(!client.is_paused());
}

#[test]
fn test_is_paused_after_pause() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.pause();
    assert!(client.is_paused());
}

#[test]
fn test_is_paused_after_unpause() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.pause();
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_get_pool_count_uninitialized() {
    let env = setup_env();
    let id = env.register_contract(None, StellarRoute);
    let client = StellarRouteClient::new(&env, &id);
    assert_eq!(client.get_pool_count(), 0);
}

#[test]
fn test_get_pool_count_after_init() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    assert_eq!(client.get_pool_count(), 0);
}

#[test]
fn test_get_pool_count_increments() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool1 = deploy_mock_pool(&env);
    let pool2 = deploy_mock_pool(&env);
    client.register_pool(&pool1);
    assert_eq!(client.get_pool_count(), 1);
    client.register_pool(&pool2);
    assert_eq!(client.get_pool_count(), 2);
}

#[test]
fn test_is_pool_registered_unknown() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    assert!(!client.is_pool_registered(&pool));
}

#[test]
fn test_is_pool_registered_after_register() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    assert!(client.is_pool_registered(&pool));
}

#[test]
fn test_is_pool_registered_different_pool() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool1 = deploy_mock_pool(&env);
    let pool2 = deploy_mock_pool(&env);
    client.register_pool(&pool1);
    assert!(client.is_pool_registered(&pool1));
    assert!(!client.is_pool_registered(&pool2));
}

// ── MEV Protection Tests ──────────────────────────────────────────────────────

mod mock_manipulated {
    use super::super::types::Asset;
    use soroban_sdk::{contract, contractimpl, Env};

    /// A pool that changes reserves between calls — simulates sandwich attack.
    #[contract]
    pub struct MockManipulatedPool;

    #[contractimpl]
    impl MockManipulatedPool {
        pub fn adapter_quote(
            _e: Env,
            _in_asset: Asset,
            _out_asset: Asset,
            amount_in: i128,
        ) -> i128 {
            amount_in * 99 / 100
        }

        pub fn swap(
            _e: Env,
            _in_asset: Asset,
            _out_asset: Asset,
            amount_in: i128,
            _min_out: i128,
        ) -> i128 {
            amount_in * 99 / 100
        }

        /// Returns different reserves on each call to simulate manipulation.
        /// First call: (1B, 1B). After swap: both go UP (manipulation signal).
        pub fn get_rsrvs(e: Env) -> (i128, i128) {
            let key = soroban_sdk::symbol_short!("call_ct");
            let count: u32 = e.storage().instance().get(&key).unwrap_or(0);
            e.storage().instance().set(&key, &(count + 1));
            if count == 0 {
                (1_000_000_000, 1_000_000_000)
            } else {
                // Both reserves increased — indicates manipulation
                (1_100_000_000, 1_100_000_000)
            }
        }
    }
}

use mock_manipulated::MockManipulatedPool;

fn deploy_manipulated_pool(env: &Env) -> Address {
    env.register_contract(None, MockManipulatedPool)
}

fn default_mev_config() -> MevConfig {
    MevConfig {
        commit_threshold: 100_000,
        commit_window_ledgers: 100,
        max_swaps_per_window: 3,
        rate_limit_window: 50,
        high_impact_threshold_bps: 10,
        price_freshness_threshold_bps: 500,
    }
}

// --- Commit-Reveal Tests ---

#[test]
fn test_commit_reveal_flow() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.configure_mev(&default_mev_config());

    let sender = Address::generate(&env);
    let amount_in: i128 = 1000;
    let min_out: i128 = 0;
    let deadline: u64 = current_seq(&env) + 200;

    // Build the hash payload
    let mut payload = Bytes::new(&env);
    payload.append(&Bytes::from_slice(&env, &amount_in.to_be_bytes()));
    payload.append(&Bytes::from_slice(&env, &min_out.to_be_bytes()));
    payload.append(&Bytes::from_slice(&env, &deadline.to_be_bytes()));
    let salt = BytesN::from_array(&env, &[1u8; 32]);
    payload.append(&Bytes::from_slice(&env, &[1u8; 32]));
    let commitment_hash = env.crypto().sha256(&payload);

    // Commit
    client.commit_swap(&sender, &commitment_hash, &1000_i128);

    // Reveal and execute
    let route = make_route(&env, &pool, 1);
    let params = SwapParams {
        route,
        amount_in,
        min_amount_out: min_out,
        recipient: Address::generate(&env),
        deadline,
        not_before: 0,
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    };

    let result = client.reveal_and_execute(&sender, &params, &salt);
    assert!(result.amount_out > 0);
    assert_eq!(result.amount_in, 1000);
}

#[test]
fn test_expired_commitment() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.configure_mev(&default_mev_config());

    let sender = Address::generate(&env);
    let amount_in: i128 = 1000;
    let min_out: i128 = 0;
    let deadline: u64 = 500;

    let mut payload = Bytes::new(&env);
    payload.append(&Bytes::from_slice(&env, &amount_in.to_be_bytes()));
    payload.append(&Bytes::from_slice(&env, &min_out.to_be_bytes()));
    payload.append(&Bytes::from_slice(&env, &deadline.to_be_bytes()));
    let salt = BytesN::from_array(&env, &[2u8; 32]);
    payload.append(&Bytes::from_slice(&env, &[2u8; 32]));
    let commitment_hash = env.crypto().sha256(&payload);

    client.commit_swap(&sender, &commitment_hash, &1000_i128);

    // Advance past expiry
    env.ledger().with_mut(|li| li.sequence_number = 200);

    let route = make_route(&env, &pool, 1);
    let params = SwapParams {
        route,
        amount_in,
        min_amount_out: min_out,
        recipient: Address::generate(&env),
        deadline,
        not_before: 0,
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    };

    let result = client.try_reveal_and_execute(&sender, &params, &salt);
    assert_eq!(result, Err(Ok(ContractError::CommitmentExpired)));
}

#[test]
fn test_invalid_reveal_rejected() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.configure_mev(&default_mev_config());

    let sender = Address::generate(&env);
    // Commit with one hash
    let commitment_hash = BytesN::from_array(&env, &[99u8; 32]);
    client.commit_swap(&sender, &commitment_hash, &1000_i128);

    // Try to reveal with different params (wrong hash)
    let wrong_salt = BytesN::from_array(&env, &[88u8; 32]);
    let route = make_route(&env, &pool, 1);
    let params = SwapParams {
        route,
        amount_in: 1000,
        min_amount_out: 0,
        recipient: Address::generate(&env),
        deadline: current_seq(&env) + 200,
        not_before: 0,
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    };

    let result = client.try_reveal_and_execute(&sender, &params, &wrong_salt);
    assert_eq!(result, Err(Ok(ContractError::CommitmentNotFound)));
}

// --- Rate Limiting Tests ---

#[test]
fn test_rate_limiting_blocks_excessive_swaps() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    // Set max 3 swaps per window
    client.configure_mev(&default_mev_config());

    // First 3 swaps should succeed
    for _ in 0..3 {
        simple_swap(&env, &client, &pool);
    }

    // 4th swap from same address should fail — but simple_swap generates new addresses.
    // We need the same sender for all swaps.
    let sender = Address::generate(&env);
    let make_params = |env: &Env| {
        SwapParams {
            route: make_route(env, &pool, 1),
            amount_in: 1000,
            min_amount_out: 0,
            recipient: Address::generate(env),
            deadline: current_seq(env) + 100,
            not_before: 0,
            max_price_impact_bps: 0,
            max_execution_spread_bps: 0,
        }
    };

    // Reset with a fresh router to avoid contamination from earlier swaps
    let (_, _, client2) = deploy_router(&env);
    client2.register_pool(&pool);
    client2.configure_mev(&default_mev_config());

    for _ in 0..3 {
        client2.execute_swap(&sender, &make_params(&env));
    }

    let result = client2.try_execute_swap(&sender, &make_params(&env));
    assert_eq!(result, Err(Ok(ContractError::RateLimitExceeded)));
}

#[test]
fn test_rate_limiting_whitelisted_exempt() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.configure_mev(&default_mev_config());

    let sender = Address::generate(&env);
    client.set_whitelist(&sender, &true);

    let make_params = |env: &Env| {
        SwapParams {
            route: make_route(env, &pool, 1),
            amount_in: 1000,
            min_amount_out: 0,
            recipient: Address::generate(env),
            deadline: current_seq(env) + 100,
            not_before: 0,
            max_price_impact_bps: 0,
            max_execution_spread_bps: 0,
        }
    };

    // Should succeed even beyond the limit
    for _ in 0..5 {
        client.execute_swap(&sender, &make_params(&env));
    }
}

// --- Price Impact Tests ---

#[test]
fn test_max_price_impact_rejection() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    // 1 hop = 5 bps impact. Set max to 1 bps → should fail.
    let params = SwapParams {
        route: make_route(&env, &pool, 1),
        amount_in: 1000,
        min_amount_out: 0,
        recipient: Address::generate(&env),
        deadline: current_seq(&env) + 100,
        not_before: 0,
        max_price_impact_bps: 1,
        max_execution_spread_bps: 0,
    };

    let result = client.try_execute_swap(&Address::generate(&env), &params);
    assert_eq!(result, Err(Ok(ContractError::PriceImpactTooHigh)));
}

// --- Execution Window Tests ---

#[test]
fn test_not_before_enforcement() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    let params = SwapParams {
        route: make_route(&env, &pool, 1),
        amount_in: 1000,
        min_amount_out: 0,
        recipient: Address::generate(&env),
        deadline: current_seq(&env) + 200,
        not_before: current_seq(&env) + 100, // in the future
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    };

    let result = client.try_execute_swap(&Address::generate(&env), &params);
    assert_eq!(result, Err(Ok(ContractError::ExecutionTooEarly)));
}

#[test]
fn test_not_before_at_boundary_succeeds() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    let params = SwapParams {
        route: make_route(&env, &pool, 1),
        amount_in: 1000,
        min_amount_out: 0,
        recipient: Address::generate(&env),
        deadline: current_seq(&env) + 200,
        not_before: current_seq(&env), // exactly now
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    };

    let result = client.try_execute_swap(&Address::generate(&env), &params);
    assert!(result.is_ok());
}

#[test]
fn test_deadline_and_not_before_combined() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    env.ledger().with_mut(|li| li.sequence_number = 50);

    // Narrow window: not_before=50, deadline=60
    let params = SwapParams {
        route: make_route(&env, &pool, 1),
        amount_in: 1000,
        min_amount_out: 0,
        recipient: Address::generate(&env),
        deadline: 60,
        not_before: 50,
        max_price_impact_bps: 0,
        max_execution_spread_bps: 0,
    };

    let result = client.try_execute_swap(&Address::generate(&env), &params);
    assert!(result.is_ok());
}

// --- Commitment Required Tests ---

#[test]
fn test_commitment_required_for_large_swap() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);
    client.configure_mev(&default_mev_config()); // threshold = 100_000

    let params = swap_params_for(
        &env,
        make_route(&env, &pool, 1),
        100_000, // equals threshold
        0,
        current_seq(&env) + 100,
    );

    let result = client.try_execute_swap(&Address::generate(&env), &params);
    assert_eq!(result, Err(Ok(ContractError::CommitmentRequired)));
}

// --- Reserve Validation Tests ---

#[test]
fn test_reserve_validation_catches_manipulation() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_manipulated_pool(&env);
    client.register_pool(&pool);

    let params = swap_params_for(
        &env,
        make_route(&env, &pool, 1),
        1000,
        0,
        current_seq(&env) + 100,
    );

    let result = client.try_execute_swap(&Address::generate(&env), &params);
    assert_eq!(result, Err(Ok(ContractError::ReserveManipulationDetected)));
}

// --- Admin Config Tests ---

#[test]
fn test_configure_mev_success() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    client.configure_mev(&default_mev_config());
    let config = client.get_mev_config();
    assert_eq!(config.commit_threshold, 100_000);
    assert_eq!(config.max_swaps_per_window, 3);
}

#[test]
fn test_high_impact_swap_event_emitted() {
    let env = setup_env();
    let (_, _, client) = deploy_router(&env);
    let pool = deploy_mock_pool(&env);
    client.register_pool(&pool);

    // Set high impact threshold very low so it triggers
    let config = MevConfig {
        commit_threshold: 1_000_000,
        commit_window_ledgers: 100,
        max_swaps_per_window: 100,
        rate_limit_window: 50,
        high_impact_threshold_bps: 1, // very low, will trigger on any swap
        price_freshness_threshold_bps: 500,
    };
    client.configure_mev(&config);

    let events_before = env.events().all().len();
    simple_swap(&env, &client, &pool);
    // More events should have been emitted (including hi_imp)
    assert!(env.events().all().len() > events_before);
}
