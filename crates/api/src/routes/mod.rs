//! API routes

pub mod health;
pub mod orderbook;
pub mod pairs;
pub mod quote;

use axum::{routing::get, Router};
use std::sync::Arc;

use crate::state::AppState;

/// Create the main API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // API v1 routes
        .route("/api/v1/pairs", get(pairs::list_pairs))
        .route(
            "/api/v1/orderbook/:base/:quote",
            get(orderbook::get_orderbook),
        )
        .route("/api/v1/quote/:base/:quote", get(quote::get_quote))
        .with_state(state)
}
