use std::{fs::File, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub size_limit: Option<usize>,
    pub cache_expire: Option<u16>,
    pub track_expire: Option<u16>,
    pub track_interval: Option<u16>,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let file = File::open(path.as_ref())?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}
