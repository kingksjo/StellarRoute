#![no_std]

pub mod adapters;
pub mod constant_product_adapter;
pub mod errors;
pub mod events;
pub mod router;
pub mod storage;
pub mod types;

#[cfg(test)]
mod benchmarks;
#[cfg(test)]
mod test;
