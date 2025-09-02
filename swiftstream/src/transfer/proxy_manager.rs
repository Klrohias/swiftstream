use std::collections::HashMap;

use reqwest::Url;
use url::ParseError;

pub struct ProxyManager {
    proxies: HashMap<String, Url>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            proxies: HashMap::new(),
        }
    }

    pub fn load_proxies(&mut self, input: HashMap<String, String>) -> Result<(), ParseError> {
        let map = input
            .into_iter()
            .map(|entry| Ok((entry.0, Url::parse(&entry.1)?)))
            .collect::<Result<HashMap<String, Url>, ParseError>>()?;

        for (k, v) in map.into_iter() {
            self.proxies.insert(k, v);
        }

        Ok(())
    }

    pub fn get_proxy(&self, hostname: impl AsRef<str>) -> Option<Url> {
        for (k, v) in self.proxies.iter() {
            if k.contains(hostname.as_ref()) {
                return Some(v.clone());
            }
        }

        None
    }
}
