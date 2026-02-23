use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "PoolAdapterClient")]
pub trait PoolAdapterTrait {
    // Executes the actual swap
    fn swap(
        e: Env,
        input_asset: Address,
        output_asset: Address,
        amount_in: i128,
        min_out: i128,
    ) -> i128;

    // Preview the output (used by get_quote)
    fn preview_quote(e: Env, input_asset: Address, output_asset: Address, amount_in: i128) -> i128;

    // Fetch liquidity depth
    fn get_reserves(e: Env) -> (i128, i128);
}
