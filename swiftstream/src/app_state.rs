use std::sync::Arc;

use reqwest::{Client, Proxy};
use typed_container::Container;

use crate::{
    Config,
    caching::{CachePool, Downloader, StreamTrackingPool},
    transfer::ProxyManager,
};

pub type AppStateRef = Arc<AppState>;
pub struct AppState {
    pub config: Arc<Config>,
    pub cache_pool: Arc<CachePool>,
    pub tracking_pool: Arc<StreamTrackingPool>,
    pub http_client: Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let container = Container::new();

        container.register_constructor(|_| {
            let mut builder = Client::builder();

            if let Some(user_agent) = &config.http.user_agent {
                builder = builder.user_agent(user_agent)
            }

            if let Some(proxies) = config.http.proxies.clone() {
                let mut proxy_manager = ProxyManager::new();
                proxy_manager.load_proxies(proxies).unwrap();

                builder = builder.proxy(Proxy::custom(move |url| match url.host_str() {
                    Some(hostname) => proxy_manager.get_proxy(hostname),
                    None => None,
                }));
            }

            builder.build().unwrap()
        });

        container.register_constructor(|x| {
            Arc::new(Downloader::new(
                x.get(),
                config.download_threads.unwrap_or(1), // default single thread
            ))
        });

        container.register_constructor(|x| {
            CachePool::new(
                config.size_limit.unwrap_or(512 * 1024 * 1024), // 512MB
                config.cache_expire.unwrap_or(30),              // 30s
                x.get(),
            )
        });

        container.register_constructor(|x| {
            StreamTrackingPool::new(
                config.track_expire.unwrap_or(60),  // 60s
                config.track_interval.unwrap_or(8), // 8s
                x.get(),
                x.get(),
            )
        });

        Self {
            config: config.clone(),
            cache_pool: container.get(),
            tracking_pool: container.get(),
            http_client: container.get(),
        }
    }
}
