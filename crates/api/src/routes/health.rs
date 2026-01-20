//! Health check endpoint

use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{error::Result, models::HealthResponse, state::AppState};

/// Health check endpoint
///
/// Returns the service status and version information
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>> {
    let timestamp = chrono::Utc::now().timestamp();

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: state.version.clone(),
        timestamp,
    }))
}
