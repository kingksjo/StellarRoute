//! Integration tests for the indexer

use stellarroute_indexer::config::IndexerConfig;
use stellarroute_indexer::db::Database;
use stellarroute_indexer::horizon::HorizonClient;
use stellarroute_indexer::models::asset::Asset;
use tracing::debug;

#[tokio::test]
#[ignore] // Requires database and Horizon API
async fn test_database_connection() {
    let config = IndexerConfig {
        stellar_horizon_url: "https://horizon-testnet.stellar.org".to_string(),
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://stellarroute:stellarroute_dev@localhost:5432/stellarroute".to_string()
        }),
        poll_interval_secs: 5,
        horizon_limit: 200,
        max_connections: 5,
        min_connections: 1,
        connection_timeout_secs: 30,
        idle_timeout_secs: 600,
        max_lifetime_secs: 1800,
    };

    let db = Database::new(&config)
        .await
        .expect("Failed to connect to database");
    db.health_check().await.expect("Health check failed");
}

#[tokio::test]
#[ignore] // Requires Horizon API
async fn test_horizon_client_get_offers() {
    let client = HorizonClient::new("https://horizon-testnet.stellar.org");
    let offers = client.get_offers(Some(10), None, None).await;

    // Should succeed if Horizon API is accessible
    assert!(offers.is_ok());
    if let Ok(offers) = offers {
        // May be empty, but should be a valid response
        debug!(count = offers.len(), "Fetched offers");
    }
}

#[test]
fn test_asset_key_generation() {
    let native = Asset::Native;
    let (asset_type, code, issuer) = native.key();
    assert_eq!(asset_type, "native");
    assert_eq!(code, None);
    assert_eq!(issuer, None);

    let usdc = Asset::CreditAlphanum4 {
        asset_code: "USDC".to_string(),
        asset_issuer: "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string(),
    };
    let (asset_type, code, issuer) = usdc.key();
    assert_eq!(asset_type, "credit_alphanum4");
    assert_eq!(code, Some("USDC".to_string()));
    assert_eq!(
        issuer,
        Some("GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string())
    );
}
