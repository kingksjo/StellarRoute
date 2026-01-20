//! Database health monitoring

use sqlx::{PgPool, Row};
use tracing::{debug, info};

use crate::error::Result;

/// Database health metric
#[derive(Debug, Clone)]
pub struct HealthMetric {
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_unit: String,
}

/// Database health monitor
pub struct HealthMonitor {
    pool: PgPool,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get current database health metrics
    pub async fn get_health_metrics(&self) -> Result<Vec<HealthMetric>> {
        debug!("Fetching database health metrics");

        let rows = sqlx::query(
            r#"
            select
                metric_name,
                metric_value::float8 as metric_value,
                metric_unit
            from get_db_health_metrics()
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let metrics: Vec<HealthMetric> = rows
            .into_iter()
            .map(|row| HealthMetric {
                metric_name: row.get("metric_name"),
                metric_value: row.get("metric_value"),
                metric_unit: row.get("metric_unit"),
            })
            .collect();

        info!("Fetched {} health metrics", metrics.len());
        Ok(metrics)
    }

    /// Record a health metric
    pub async fn record_metric(
        &self,
        metric_name: &str,
        metric_value: f64,
        metric_unit: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            insert into db_health_metrics (metric_name, metric_value, metric_unit, metadata)
            values ($1, $2, $3, $4)
            "#,
        )
        .bind(metric_name)
        .bind(metric_value)
        .bind(metric_unit)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        debug!(
            "Recorded metric: {} = {} {}",
            metric_name,
            metric_value,
            metric_unit.unwrap_or("")
        );
        Ok(())
    }

    /// Get connection pool stats
    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }

    /// Check if database is healthy
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("select 1").execute(&self.pool).await.is_ok()
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

impl PoolStats {
    /// Get the number of active connections
    pub fn active(&self) -> u32 {
        self.size.saturating_sub(self.idle as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_stats_active() {
        let stats = PoolStats { size: 10, idle: 3 };
        assert_eq!(stats.active(), 7);
    }

    #[test]
    fn test_pool_stats_all_idle() {
        let stats = PoolStats { size: 10, idle: 10 };
        assert_eq!(stats.active(), 0);
    }
}
