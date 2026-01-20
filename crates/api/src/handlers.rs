//! API request handlers
//!
//! This module re-exports all route handlers for convenience.

pub use crate::routes::{
    health::health_check, orderbook::get_orderbook, pairs::list_pairs, quote::get_quote,
};
