use std::{
    collections::HashMap,
    io::Cursor,
    sync::Arc,
    time::{Duration, SystemTime},
};

use log::{debug, warn};
use mediastream_rs::format::M3uPlaylist;
use reqwest::Client;
use tokio::{sync::RwLock, time::sleep};
use url::Url;

use crate::{caching::CachePool, transfer::parse_m3u8_async};

pub struct StreamTrackingPool {
    tracking: RwLock<HashMap<String, Arc<TrackingItem>>>,
    time_limit_secs: u16,
    interval: Duration,
    cache_pool: Arc<CachePool>,
    http_client: Arc<Client>,
}

impl StreamTrackingPool {
    pub fn new(
        time_limit_secs: u16,
        interval_secs: u16,
        cache_pool: Arc<CachePool>,
        http_client: Arc<Client>,
    ) -> Arc<Self> {
        Arc::new(Self {
            tracking: RwLock::new(HashMap::new()),
            time_limit_secs,
            interval: Duration::from_secs(interval_secs.into()),
            cache_pool,
            http_client,
        })
    }

    async fn get_internal(self: &Arc<Self>, origin: String) -> Arc<TrackingItem> {
        if let Some(item_ref) = self.tracking.read().await.get(&origin) {
            return item_ref.clone();
        }

        // new cache item
        let result = Arc::new(TrackingItem::new(origin.clone()));
        self.tracking.write().await.insert(origin, result.clone());

        // worker startup
        let worker_result_ref = result.clone();
        let worker_self_ref = self.clone();
        tokio::spawn(async move {
            worker_result_ref.manage_worker(worker_self_ref).await;
        });

        result
    }

    pub async fn drop(self: &Arc<Self>, origin: impl AsRef<str>) {
        self.tracking.write().await.remove(origin.as_ref());
    }

    pub async fn track(self: &Arc<Self>, origin: impl AsRef<str>) {
        let item = self.get_internal(origin.as_ref().to_owned()).await;
        item.set_expire(SystemTime::now() + Duration::from_secs(self.time_limit_secs.into()))
            .await;
    }

    async fn cache_prepare(&self, origin: impl AsRef<str>) {
        self.cache_pool.prepare(origin).await;
    }
}

struct TrackingItem {
    origin: String,
    expire: RwLock<SystemTime>,
}

impl TrackingItem {
    pub fn new(origin: String) -> Self {
        Self {
            origin,
            expire: RwLock::new(SystemTime::now() + Duration::from_secs(30)),
        }
    }

    pub async fn manage_worker(&self, tracking_pool: Arc<StreamTrackingPool>) {
        loop {
            let expire = self.expire.read().await.clone();
            let now = SystemTime::now();
            if expire < now {
                // expired
                break;
            }

            // keep track
            if let Err(e) = self.keep_track(&tracking_pool).await {
                warn!("Error while keep track of {}: {}", self.origin, e);
            }
            debug!("Kept track of {}", self.origin);

            // or wait...
            tokio::select! {
                _ = sleep(expire.duration_since(now).unwrap()) => {},
                _ = sleep(tracking_pool.interval) => {},
            }
        }

        // expired, drop my self
        tracking_pool.drop(&self.origin).await;
    }

    async fn prepare_all(
        &self,
        tracking_pool: &Arc<StreamTrackingPool>,
        origin: impl AsRef<str>,
        playlist: M3uPlaylist,
    ) -> Result<(), anyhow::Error> {
        let base_url = Url::parse(origin.as_ref())?;

        // prepare all
        for media in playlist.medias.iter() {
            let media_location = media.location.clone();
            let mut location = Url::parse(&media_location);
            if location == Err(url::ParseError::RelativeUrlWithoutBase) {
                location = base_url.join(&media_location);
            }
            let location = location?.to_string();

            tracking_pool.cache_prepare(location).await;
        }

        Ok(())
    }

    async fn keep_track(
        &self,
        tracking_pool: &Arc<StreamTrackingPool>,
    ) -> Result<(), anyhow::Error> {
        let data = tracking_pool
            .http_client
            .get(&self.origin)
            .send()
            .await?
            .bytes()
            .await?;

        // parse
        let playlist = parse_m3u8_async(Cursor::new(data)).await?;

        self.prepare_all(tracking_pool, &self.origin, playlist)
            .await?;

        Ok(())
    }

    pub async fn set_expire(&self, expire: SystemTime) {
        let mut expire_ref = self.expire.write().await;
        *expire_ref = expire;
    }
}
