use std::{collections::HashMap, fs::File, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub listen_addr: String,
    pub limit: Option<usize>,
    pub upstreams: HashMap<String, UpstreamConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamConfig {
    pub url: String,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let file = File::open(path.as_ref())?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}
