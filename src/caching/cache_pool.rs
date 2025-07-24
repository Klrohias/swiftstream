use log::error;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    io,
    sync::{Mutex, RwLock, oneshot},
    task::yield_now,
    time::sleep,
};

use crate::caching::download;

pub struct CachePool {
    cached: RwLock<HashMap<String, Arc<CacheItem>>>,
    space_limit: usize,
}

#[derive(Clone, Debug)]
pub struct CacheResult {
    pub bytes: Arc<[u8]>,
    pub content_type: String,
}

impl CachePool {
    pub fn new(space_limit: usize) -> Arc<Self> {
        Arc::new(CachePool {
            cached: RwLock::new(HashMap::new()),
            space_limit,
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

        if self.get_total_size().await > self.space_limit {
            return None;
        }

        // new cache item
        let result = Arc::new(CacheItem::new(origin.clone()));
        self.cached.write().await.insert(origin, result.clone());

        // worker startup
        let worker_result_ref = result.clone();
        let worker_self_ref = self.clone();
        tokio::spawn(async move {
            worker_result_ref.manage_worker(worker_self_ref).await;
        });

        Some(result)
    }

    pub async fn get(self: &Arc<Self>, origin: impl AsRef<str>) -> Result<CacheResult, io::Error> {
        let cache_item = self.get_internal(origin.as_ref().to_owned()).await;
        if cache_item.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Space limit reached",
            ));
        }
        let cache_item = cache_item.unwrap();

        cache_item
            .set_expire(SystemTime::now() + Duration::from_secs(30))
            .await;

        yield_now().await;

        let data = cache_item.get_resource().await.ok_or(io::Error::new(
            io::ErrorKind::Interrupted,
            "Failed to load data",
        ))?;

        Ok(data)
    }

    pub async fn drop(self: &Arc<Self>, origin: impl AsRef<str>) {
        self.cached.write().await.remove(origin.as_ref());
    }
}

struct CacheItem {
    data: RwLock<Option<CacheResult>>,
    origin: String,
    expire: RwLock<SystemTime>,
    cancel_tx: Mutex<Option<oneshot::Sender<()>>>,
}

impl CacheItem {
    pub fn new(origin: String) -> Self {
        Self {
            data: RwLock::new(None),
            origin,
            expire: RwLock::new(SystemTime::now() + Duration::from_secs(30)),
            cancel_tx: Mutex::new(None),
        }
    }

    pub async fn manage_worker(&self, cache_pool: Arc<CachePool>) {
        let (tx, rx) = oneshot::channel();
        {
            let mut cancel_tx = self.cancel_tx.lock().await;
            *cancel_tx = Some(tx);
        }

        self.manage_worker_internal(cache_pool, rx).await;
    }

    async fn manage_worker_internal(
        &self,
        cache_pool: Arc<CachePool>,
        cancel_rx: oneshot::Receiver<()>,
    ) {
        tokio::select! {
            _ = cancel_rx => {},
            _ = async {
                    self.load_resource().await;
                    // wait for expire
                    self.wait_for_expire().await;
            } => {}
        };

        // expired, drop my self
        cache_pool.drop(&self.origin).await;
    }

    async fn load_resource(&self) {
        let mut write_lock = self.data.write().await;
        loop {
            let expire = self.expire.read().await.clone();
            let now = SystemTime::now();
            if expire < now {
                // expired
                break;
            }

            // try load
            match self.try_load_resource().await {
                Err(e) => {
                    error!("Error while load resource {}: {}", self.origin, e);
                }
                Ok(s) => {
                    *write_lock = Some(s);
                    break; // break and release the writer lock
                }
            }
        }
    }

    async fn try_load_resource(&self) -> Result<CacheResult, anyhow::Error> {
        let (bytes, content_type) = download(&self.origin).await?;

        Ok(CacheResult {
            bytes,
            content_type,
        })
    }

    async fn wait_for_expire(&self) {
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

    pub async fn get_resource(&self) -> Option<CacheResult> {
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
