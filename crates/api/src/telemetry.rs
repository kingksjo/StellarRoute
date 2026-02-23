//! Structured logging initialisation for the API server.
//!
//! Reads `RUST_LOG` for the filter and `LOG_FORMAT` to choose the output
//! format.  Both environment variables are optional.
//!
//! # Environment variables
//!
//! | Variable     | Values              | Default  |
//! |-------------|---------------------|----------|
//! | `RUST_LOG`  | tracing filter spec | `info`   |
//! | `LOG_FORMAT`| `json` \| `pretty`  | `pretty` |
//!
//! ## Examples
//!
//! ```bash
//! # Development — human-readable output, debug level for this crate
//! RUST_LOG=stellarroute_api=debug ./stellarroute-api
//!
//! # Production — structured JSON, info level
//! RUST_LOG=info LOG_FORMAT=json ./stellarroute-api
//! ```

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialise the global tracing subscriber.
///
/// Call **once** at the very start of `main`, before any other code runs,
/// so that every log event is captured by the configured subscriber.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match std::env::var("LOG_FORMAT")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "json" => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init(),
        _ => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init(),
    }
}
