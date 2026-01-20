//! API server setup and configuration

use axum::Router;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{docs::ApiDoc, error::Result, middleware::RateLimitLayer, routes, state::AppState};

/// API server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_cors: true,
        }
    }
}

/// API Server
pub struct Server {
    config: ServerConfig,
    app: Router,
}

impl Server {
    /// Create a new API server
    pub fn new(config: ServerConfig, db: PgPool) -> Self {
        let state = Arc::new(AppState::new(db));
        let app = Self::build_app(state, &config);

        Self { config, app }
    }

    /// Build the application router
    fn build_app(state: Arc<AppState>, config: &ServerConfig) -> Router {
        let mut app = routes::create_router(state);

        // Add Swagger UI for API documentation
        let swagger =
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
        app = app.merge(swagger);

        // Add CORS if enabled
        if config.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            app = app.layer(cors);
        }

        // Add rate limiting
        let rate_limit = RateLimitLayer::default();
        app = app.layer(rate_limit);

        app
    }

    /// Start the server
    pub async fn start(self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .expect("Invalid socket address");

        info!("🚀 StellarRoute API server starting on http://{}", addr);
        info!("📊 Health check: http://{}/health", addr);
        info!("📈 Trading pairs: http://{}/api/v1/pairs", addr);
        info!("📚 API Documentation: http://{}/swagger-ui", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind address");

        axum::serve(listener, self.app).await.expect("Server error");

        Ok(())
    }

    /// Get router for testing
    #[cfg(test)]
    pub fn router(self) -> Router {
        self.app
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert!(config.enable_cors);
    }
}
