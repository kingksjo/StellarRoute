//! Rate limiting middleware

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tower::{Layer, Service};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

/// Rate limiter state
#[derive(Clone)]
struct RateLimiterState {
    requests: HashMap<IpAddr, Vec<Instant>>,
}

impl RateLimiterState {
    fn new() -> Self {
        Self {
            requests: HashMap::new(),
        }
    }

    fn check_rate_limit(&mut self, ip: IpAddr, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let cutoff = now - config.window;

        // Get or create request history for this IP
        let requests = self.requests.entry(ip).or_insert_with(Vec::new);

        // Remove old requests
        requests.retain(|&time| time > cutoff);

        // Check if under limit
        if requests.len() < config.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Rate limiting layer
#[derive(Clone)]
pub struct RateLimitLayer {
    state: Arc<Mutex<RateLimiterState>>,
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState::new())),
            config,
        }
    }
}

impl Default for RateLimitLayer {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            state: self.state.clone(),
            config: self.config.clone(),
        }
    }
}

/// Rate limiting service
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    state: Arc<Mutex<RateLimiterState>>,
    config: RateLimitConfig,
}

impl<S> Service<Request> for RateLimitService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Extract IP address (simplified - would need proper forwarded header handling)
            let ip = IpAddr::from([127, 0, 0, 1]); // Default to localhost

            // Check rate limit
            let mut state = state.lock().await;
            let allowed = state.check_rate_limit(ip, &config);
            drop(state);

            if !allowed {
                return Ok((
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded. Please try again later.",
                )
                    .into_response());
            }

            inner.call(req).await
        })
    }
}

/// Middleware function for rate limiting
pub async fn rate_limit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // This is a simplified version - the Layer approach above is more robust
    Ok(next.run(req).await)
}
