use soroban_sdk::{contracttype, Address, Symbol, Vec};

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
#[derive(Clone, Debug)]
pub struct RouteHop {
    pub source: Asset,
    pub destination: Asset,
    pub pool: Address,
    pub pool_type: PoolType,
}

#[contracttype]
#[derive(Clone, Debug)]
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
}
