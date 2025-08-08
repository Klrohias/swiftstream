//! A simple container for IoC
//! Example:
//! ```
//! let config = /* ... */;
//! let container = Container::new();
//! container.register_constructor(|_| Arc::new(Client::new()));
//!
//! let config_cloned = config.clone();
//! container.register_constructor(move |x| {
//!     Arc::new(Downloader::new(
//!         x.get(),
//!         config_cloned.download_threads.unwrap_or(1), // default single thread
//!     ))
//! });
//!
//! let config_cloned = config.clone();
//! container.register_constructor(move |x| {
//!     CachePool::new(
//!         config_cloned.size_limit.unwrap_or(512 * 1024 * 1024), // 512MB
//!         config_cloned.cache_expire.unwrap_or(30),              // 30s
//!         x.get(),
//!     )
//! });
//!
//! let config_cloned = config.clone();
//! container.register_constructor(move |x| {
//!     StreamTrackingPool::new(
//!         config_cloned.track_expire.unwrap_or(60),  // 60s
//!         config_cloned.track_interval.unwrap_or(8), // 8s
//!         x.get(),
//!         x.get(),
//!     )
//! });
//!
//! let _cache_pool = container::get::<Arc<CachePool>>();
//! // ...
//! ```

mod container;
mod errors;
mod macros;
pub use container::*;
pub use errors::*;
