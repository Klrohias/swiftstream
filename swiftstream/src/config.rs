use std::{collections::HashMap, fs::File, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub listen_addr: String,
    pub base_url: Option<String>,
    pub size_limit: Option<usize>,
    pub cache_expire: Option<u16>,
    pub track_expire: Option<u16>,
    pub track_interval: Option<u16>,
    pub download_threads: Option<u8>,

    #[serde(default)]
    pub http: HttpConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct HttpConfig {
    pub proxies: Option<HashMap<String, String>>,
    pub user_agent: Option<String>,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let file = File::open(path.as_ref())?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}
