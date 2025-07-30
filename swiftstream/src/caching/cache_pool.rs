use log::{debug, error, warn};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{io, sync::RwLock, task::yield_now, time::sleep};

use crate::caching::{DownloadError, Downloader};

pub struct CachePool {
    cached: RwLock<HashMap<String, Arc<CacheItem>>>,
    time_limit_secs: u16,
    size_limit: usize,
    downloader: Arc<Downloader>,
}

impl CachePool {
    pub fn new(size_limit: usize, time_limit_secs: u16, downloader: Arc<Downloader>) -> Arc<Self> {
        Arc::new(CachePool {
            cached: RwLock::new(HashMap::new()),
            size_limit,
            time_limit_secs,
            downloader,
        })
    }

    pub async fn prepare(self: &Arc<Self>, origin: impl AsRef<str>) {
        let origin = origin.as_ref().to_owned();
        let self_arc = self.clone();
        tokio::spawn(async move {
            self_arc.get_internal(origin).await;
        });
    }

    async fn get_total_size(&self) -> usize {
        self.cached
            .read()
            .await
            .iter()
            .map(|x| x.1.get_size())
            .sum()
    }

    async fn get_internal(self: &Arc<Self>, origin: String) -> Option<Arc<CacheItem>> {
        if let Some(item_ref) = self.cached.read().await.get(&origin) {
            return Some(item_ref.clone());
        }

        if self.get_total_size().await > self.size_limit {
            return None;
        }

        // new cache item
        let result = Arc::new(CacheItem::new(origin.clone()));
        self.cached.write().await.insert(origin, result.clone());

        // worker startup
        let worker_item_ref = result.clone();
        let worker_self_ref = self.clone();
        tokio::spawn(async move {
            worker_self_ref.item_lifetime(worker_item_ref).await;
        });

        Some(result)
    }

    pub async fn get(
        self: &Arc<Self>,
        origin: impl AsRef<str>,
    ) -> Result<CacheResource, io::Error> {
        let cache_item = self.get_internal(origin.as_ref().to_owned()).await;
        if cache_item.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Space limit reached",
            ));
        }
        let cache_item = cache_item.unwrap();

        cache_item
            .set_expire(SystemTime::now() + Duration::from_secs(self.time_limit_secs.into()))
            .await;

        yield_now().await;

        let data = cache_item.get_resource().await.ok_or(io::Error::new(
            io::ErrorKind::Interrupted,
            "Failed to load data",
        ))?;

        Ok(data)
    }

    async fn drop(self: &Arc<Self>, origin: impl AsRef<str>) {
        self.cached.write().await.remove(origin.as_ref());
        debug!("Resource {} dropped", origin.as_ref());
    }

    async fn item_lifetime(self: &Arc<Self>, cache_item: Arc<CacheItem>) {
        let cache_item = cache_item.as_ref();

        // load resource
        self.load_item_resource(cache_item).await;

        // wait for expire
        cache_item.wait_expire().await;

        // drop the cache, finish
        self.drop(&cache_item.origin).await;
    }

    async fn load_item_resource(self: &Arc<Self>, cache_item: &CacheItem) {
        let mut write_lock = cache_item.data.write().await;
        tokio::select! {
            _ = cache_item.wait_expire() => {},
            _ = async {
                match self.try_load_item_resource(cache_item).await {
                    Err(e) => {
                        error!("Error while load resource {}: {}", cache_item.origin, e);
                    }
                    Ok(s) => {
                        *write_lock = Some(s);
                        return; // return and release the writer lock
                    }
                }
            } => {}
        };
    }

    async fn try_load_item_resource(
        self: &Arc<Self>,
        cache_item: &CacheItem,
    ) -> Result<CacheResource, DownloadError> {
        let origin = &cache_item.origin;
        debug!("Start downloading for {}", origin);

        // first download, with default thread count
        let download_result = self.downloader.download(origin, None).await;
        let (bytes, content_type) = match download_result {
            Ok(v) => v,
            Err(e) => {
                if !e.is_range_not_supported() && !e.is_content_length_missing() {
                    return Err(e);
                }

                warn!(
                    "Range not supported (err: {}), fallback to single-threaded for {}",
                    e, origin
                );

                // range not supported by server, try download with single thread,
                // and return error if error occurred in this time
                self.downloader.download(origin, Some(1)).await?
            }
        };
        debug!(
            "Finish downloading for {} with mime {}",
            origin, content_type
        );

        Ok(CacheResource {
            bytes,
            content_type,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CacheResource {
    pub bytes: Arc<[u8]>,
    pub content_type: String,
}

struct CacheItem {
    data: RwLock<Option<CacheResource>>,
    origin: String,
    expire: RwLock<SystemTime>,
}

impl CacheItem {
    pub fn new(origin: String) -> Self {
        Self {
            data: RwLock::new(None),
            origin,
            expire: RwLock::new(SystemTime::now() + Duration::from_secs(30)),
        }
    }

    pub async fn wait_expire(&self) {
        loop {
            let expire = self.expire.read().await.clone();
            let now = SystemTime::now();
            if expire < now {
                // expired
                break;
            }

            // or wait...
            sleep(expire.duration_since(now).unwrap()).await;
        }
    }

    pub async fn set_expire(&self, expire: SystemTime) {
        let mut expire_ref = self.expire.write().await;
        *expire_ref = expire;
    }

    pub async fn get_resource(&self) -> Option<CacheResource> {
        // wait until loaded (writer lock will release then, so we can just await the reader lock)
        let data = self.data.read().await;
        data.as_ref().map(|x| x.clone())
    }

    pub fn get_size(&self) -> usize {
        match self.data.try_read() {
            Err(_) => 0,
            Ok(lock) => lock.as_ref().map(|x| x.bytes.len()).unwrap_or(0),
        }
    }
}
