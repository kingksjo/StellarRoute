#![no_std]

pub mod adapters;
pub mod constant_product_adapter;
pub mod errors;
pub mod events;
pub mod governance;
pub mod router;
pub mod storage;
pub mod tokens;
pub mod types;
pub mod upgrade;

#[cfg(test)]
mod test;
