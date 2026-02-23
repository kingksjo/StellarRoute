use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "PoolAdapterClient")]
pub trait PoolAdapterTrait {
    fn swap(
        e: Env,
        input_asset: Address,
        output_asset: Address,
        amount_in: i128,
        min_out: i128,
    ) -> i128;

    // RENAME THIS from get_quote to adapter_quote
    fn adapter_quote(e: Env, input_asset: Address, output_asset: Address, amount_in: i128) -> i128;

    fn get_reserves(e: Env) -> (i128, i128);
}
