use std::sync::Arc;

use reqwest::Client;

use crate::{Config, caching::CachePool};

pub type AppStateRef = Arc<AppState>;
pub struct AppState {
    pub config: Config,
    pub cache_pool: Arc<CachePool>,
    /// For request the m3u file, instead of request cached ts
    pub http_client: Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let space_limit = config.limit.unwrap_or(512 * 1024 * 1024); // 512MB
        Self {
            config,
            cache_pool: CachePool::new(space_limit),
            http_client: Client::new(),
        }
    }
}
