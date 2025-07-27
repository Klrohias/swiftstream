use std::sync::Arc;

use reqwest::Client;
use typed_container::Container;

use crate::{
    Config,
    caching::{CachePool, Downloader, StreamTrackingPool},
};

pub type AppStateRef = Arc<AppState>;
pub struct AppState {
    pub config: Arc<Config>,
    pub cache_pool: Arc<CachePool>,
    pub tracking_pool: Arc<StreamTrackingPool>,
    pub http_client: Arc<Client>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let container = Container::new();
        container.register_constructor(|_| Arc::new(Client::new()));

        let config_cloned = config.clone();
        container.register_constructor(move |x| {
            Arc::new(Downloader::new(
                x.get(),
                config_cloned.download_threads.unwrap_or(1), // default single thread
            ))
        });

        let config_cloned = config.clone();
        container.register_constructor(move |x| {
            CachePool::new(
                config_cloned.size_limit.unwrap_or(512 * 1024 * 1024), // 512MB
                config_cloned.cache_expire.unwrap_or(30),              // 30s
                x.get(),
            )
        });

        let config_cloned = config.clone();
        container.register_constructor(move |x| {
            StreamTrackingPool::new(
                config_cloned.track_expire.unwrap_or(60),  // 60s
                config_cloned.track_interval.unwrap_or(8), // 8s
                x.get(),
                x.get(),
            )
        });

        Self {
            config,
            cache_pool: container.get(),
            tracking_pool: container.get(),
            http_client: container.get(),
        }
    }
}
