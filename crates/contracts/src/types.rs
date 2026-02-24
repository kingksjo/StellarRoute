use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
    Native,                  // XLM
    Issued(Address, Symbol), // (issuer, code)
    Soroban(Address),        // Soroban token contract address
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolType {
    Sdex,
    AmmConstProd,
    AmmStable,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct RouteHop {
    pub source: Asset,
    pub destination: Asset,
    pub pool: Address,
    pub pool_type: PoolType,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub hops: Vec<RouteHop>,
    pub estimated_output: i128,
    pub min_output: i128,
    pub expires_at: u64,
}

#[contracttype]
pub struct SwapParams {
    pub route: Route,
    pub amount_in: i128,
    pub min_amount_out: i128,
    pub recipient: Address,
    pub deadline: u64,
    pub not_before: u64,
    pub max_price_impact_bps: u32,
    pub max_execution_spread_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct QuoteResult {
    pub expected_output: i128,
    pub price_impact_bps: u32, // 100 = 1%
    pub fee_amount: i128,
    pub route: Route,
    pub valid_until: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub route: Route,
    pub executed_at: u64,
}

// --- MEV Protection Types ---

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CommitmentData {
    pub sender: Address,
    pub deposit_amount: i128,
    pub created_at: u32,
    pub expires_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MevConfig {
    pub commit_threshold: i128,
    pub commit_window_ledgers: u32,
    pub max_swaps_per_window: u32,
    pub rate_limit_window: u32,
    pub high_impact_threshold_bps: u32,
    pub price_freshness_threshold_bps: u32,
}

// Interface for AMM pools (SEP-like standard)
pub trait LiquidityPoolInterface {
    fn get_rsrvs(e: Env) -> (i128, i128);
    fn swap_out(e: Env, in_asset: Asset, out_asset: Asset, amount_in: i128) -> i128;
}

