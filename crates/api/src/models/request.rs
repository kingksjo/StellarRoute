//! API request models

use serde::Deserialize;

/// Query parameters for quote endpoint
#[derive(Debug, Deserialize)]
pub struct QuoteParams {
    /// Amount to trade
    pub amount: Option<String>,
    /// Type of quote (buy or sell)
    #[serde(default = "default_quote_type")]
    pub quote_type: QuoteType,
}

fn default_quote_type() -> QuoteType {
    QuoteType::Sell
}

/// Type of quote requested
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum QuoteType {
    /// Selling the base asset
    Sell,
    /// Buying the base asset
    Buy,
}

/// Asset identifier in path parameters
#[derive(Debug, Deserialize)]
pub struct AssetPath {
    /// Asset code (e.g., "XLM", "USDC", or "native" for XLM)
    pub asset_code: String,
    /// Asset issuer (optional, only for issued assets)
    pub asset_issuer: Option<String>,
}

impl AssetPath {
    /// Parse asset identifier from path segment
    /// Format: "native" or "CODE" or "CODE:ISSUER"
    pub fn parse(s: &str) -> Result<Self, String> {
        if s == "native" {
            return Ok(Self {
                asset_code: "native".to_string(),
                asset_issuer: None,
            });
        }

        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            1 => Ok(Self {
                asset_code: parts[0].to_uppercase(),
                asset_issuer: None,
            }),
            2 => Ok(Self {
                asset_code: parts[0].to_uppercase(),
                asset_issuer: Some(parts[1].to_string()),
            }),
            _ => Err(format!("Invalid asset format: {}", s)),
        }
    }

    /// Convert to asset type for database queries
    pub fn to_asset_type(&self) -> String {
        if self.asset_code == "native" {
            "native".to_string()
        } else {
            "credit_alphanum4".to_string() // Simplified, would need to detect alphanum12
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_native_asset() {
        let asset = AssetPath::parse("native").unwrap();
        assert_eq!(asset.asset_code, "native");
        assert_eq!(asset.asset_issuer, None);
    }

    #[test]
    fn test_parse_code_only() {
        let asset = AssetPath::parse("USDC").unwrap();
        assert_eq!(asset.asset_code, "USDC");
        assert_eq!(asset.asset_issuer, None);
    }

    #[test]
    fn test_parse_code_and_issuer() {
        let asset =
            AssetPath::parse("USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5")
                .unwrap();
        assert_eq!(asset.asset_code, "USDC");
        assert_eq!(
            asset.asset_issuer.as_deref(),
            Some("GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5")
        );
    }
}
