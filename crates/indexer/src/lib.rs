//! StellarRoute Indexer
//!
//! This crate provides the indexing service for SDEX orderbooks and Soroban AMM pools.

pub mod error;
pub mod sdex;
pub mod soroban;

/// Indexer service
pub struct Indexer {
    // TODO: Implement indexer state
}

impl Indexer {
    /// Create a new indexer instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self::new()
    }
}
