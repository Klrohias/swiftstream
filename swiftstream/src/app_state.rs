use std::sync::Arc;

use reqwest::Client;

use crate::{
    Config,
    caching::{CachePool, StreamTrackingPool},
};

pub type AppStateRef = Arc<AppState>;
pub struct AppState {
    pub config: Config,
    pub cache_pool: Arc<CachePool>,
    pub tracking_pool: Arc<StreamTrackingPool>,
    /// For request the m3u file, instead of request cached ts
    pub http_client: Arc<Client>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let http_client = Arc::new(Client::new());
        let cache_pool = CachePool::new(
            config.size_limit.unwrap_or(512 * 1024 * 1024), // 512MB
            config.cache_expire.unwrap_or(30),              // 30s
        );
        let tracking_pool = StreamTrackingPool::new(
            config.track_expire.unwrap_or(60),  // 60s
            config.track_interval.unwrap_or(5), // 5s
            cache_pool.clone(),
            http_client.clone(),
        );

        Self {
            config,
            cache_pool,
            tracking_pool,
            http_client,
        }
    }
}
