use std::{error::Error, fmt::Display, sync::Arc};

use reqwest::{Client, header};

#[derive(Debug)]
pub enum DownloadError {
    RequestError(reqwest::Error),
    RequestNotSuccess(u16),
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError(e) => e.fmt(f),
            Self::RequestNotSuccess(status_code) => {
                write!(f, "Server respond with status code {}", status_code)
            }
        }
    }
}

impl Error for DownloadError {}

impl From<reqwest::Error> for DownloadError {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError(value)
    }
}

pub async fn download(origin: impl AsRef<str>) -> Result<(Arc<[u8]>, String), DownloadError> {
    let response = Client::new().get(origin.as_ref()).send().await?;

    if !response.status().is_success() {
        return Err(DownloadError::RequestNotSuccess(response.status().as_u16()));
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|x| x.to_str().unwrap_or("application/octet-stream").to_owned())
        .unwrap_or("application/octet-stream".to_string());

    let bytes = response.bytes().await?;

    Ok((Arc::from(bytes.as_ref()), content_type))
}
