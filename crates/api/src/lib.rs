//! StellarRoute API Server
//!
//! Provides REST API endpoints for price quotes and orderbook data.

pub mod error;
pub mod handlers;
pub mod server;

/// API service
pub struct ApiServer {
    // TODO: Implement API server
}

impl ApiServer {
    /// Create a new API server instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ApiServer {
    fn default() -> Self {
        Self::new()
    }
}
