//! StellarRoute Indexer Binary
//!
//! Main entry point for the SDEX orderbook indexer service.

use std::process;
use tracing::{error, info};

use stellarroute_indexer::config::IndexerConfig;
use stellarroute_indexer::db::Database;
use stellarroute_indexer::horizon::HorizonClient;
use stellarroute_indexer::sdex::SdexIndexer;

#[tokio::main]
async fn main() {
    // Initialize structured logging (reads RUST_LOG and LOG_FORMAT env vars)
    stellarroute_indexer::telemetry::init();

    info!("Starting StellarRoute Indexer");

    // Load configuration
    let config = match IndexerConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Initialize database
    let db = match Database::new(&config).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = db.migrate().await {
        error!("Failed to run migrations: {}", e);
        process::exit(1);
    }

    // Initialize Horizon client
    let horizon = HorizonClient::new(&config.stellar_horizon_url);

    // Create indexer
    let indexer = SdexIndexer::new(horizon, db);

    // Start indexing
    info!("Starting SDEX indexing loop");
    if let Err(e) = indexer.start_indexing().await {
        error!("Indexer error: {}", e);
        process::exit(1);
    }
}
