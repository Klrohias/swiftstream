use std::{error::Error, fmt::Display, sync::Arc};

use bytes::{BufMut, BytesMut};
use futures::future::join_all;
use reqwest::{Client, header};

pub struct Downloader {
    http_client: Arc<Client>,
    default_threads: u8,
}

impl Downloader {
    pub fn new(http_client: Arc<Client>, default_threads: u8) -> Self {
        Downloader {
            http_client,
            default_threads,
        }
    }

    pub fn set_default_threads(&mut self, default_threads: u8) {
        self.default_threads = default_threads;
    }

    pub fn get_default_threads(&mut self) -> u8 {
        self.default_threads
    }

    async fn download_single_thread(
        &self,
        origin: impl AsRef<str>,
    ) -> Result<(Arc<[u8]>, String), DownloadError> {
        let response = self.http_client.get(origin.as_ref()).send().await?;
        if !response.status().is_success() {
            return Err(DownloadError::RequestNotSuccess(response.status().as_u16()));
        }
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|x| x.to_str().unwrap_or("application/octet-stream"))
            .unwrap_or("application/octet-stream")
            .to_owned();

        let bytes = response.bytes().await?;

        return Ok((Arc::from(bytes.as_ref()), content_type));
    }

    async fn download_range(
        http_client: Arc<Client>,
        origin: impl AsRef<str>,
        start: u64,
        end: u64,
    ) -> Result<(u64, bytes::Bytes), DownloadError> {
        let range_header = format!("bytes={}-{}", start, end);
        let response = http_client
            .get(origin.as_ref())
            .header(header::RANGE, range_header)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DownloadError::RequestNotSuccess(response.status().as_u16()));
        }

        let bytes = response.bytes().await?;
        Ok((start, bytes))
    }

    pub async fn download(
        &self,
        origin: impl AsRef<str>,
        threads: Option<u8>,
    ) -> Result<(Arc<[u8]>, String), DownloadError> {
        let threads = threads.unwrap_or(self.default_threads);
        let origin = origin.as_ref();

        if threads <= 1 {
            // Fallback to single-threaded download if only one thread is requested
            return self.download_single_thread(origin).await;
        }

        // Step 1: Get Content-Length
        let head_response = self.http_client.head(origin).send().await?;
        if !head_response.status().is_success() {
            return Err(DownloadError::RequestNotSuccess(
                head_response.status().as_u16(),
            ));
        }

        let content_length = head_response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse::<u64>().ok())
            .ok_or(DownloadError::ContentLengthMissing)?;

        // Check if server supports range requests
        if !head_response.headers().contains_key(header::ACCEPT_RANGES) {
            // Some servers might not explicitly send Accept-Ranges but still support it.
            // For simplicity, we'll assume it's not supported if the header is absent.
            // A more robust solution might involve trying a small range request first.
            // For now, if num_threads > 1 and Accept-Ranges is missing, we'll error out.
            if threads > 1 {
                return Err(DownloadError::RangeNotSupported);
            }
        }

        let content_type = head_response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|x| x.to_str().unwrap_or("application/octet-stream").to_owned())
            .unwrap_or("application/octet-stream".to_string());

        let chunk_size = content_length / threads as u64;
        let mut tasks = Vec::with_capacity(threads.into());

        for i in 0..threads {
            let start = i as u64 * chunk_size;
            let end = if i == threads - 1 {
                content_length - 1
            } else {
                start + chunk_size - 1
            };
            let client_clone = self.http_client.clone();
            let origin_clone = origin.to_string();

            tasks.push(tokio::spawn(async move {
                Self::download_range(client_clone, origin_clone, start, end).await
            }));
        }

        let results = join_all(tasks).await;

        let mut downloaded_parts: Vec<(u64, bytes::Bytes)> = Vec::with_capacity(threads.into());
        for res in results {
            match res {
                Ok(Ok(part)) => downloaded_parts.push(part),
                Ok(Err(e)) => return Err(e), // Propagate download errors
                Err(_) => return Err(DownloadError::ReassemblyError), // Task join error
            }
        }

        // Sort parts by their starting byte to reassemble correctly
        downloaded_parts.sort_by_key(|(start, _)| *start);

        let mut buffer = BytesMut::with_capacity(content_length as usize);
        let mut current_pos = 0;

        for (start, bytes) in downloaded_parts {
            if start != current_pos {
                // This indicates a gap or out-of-order chunk, which is an error in reassembly
                return Err(DownloadError::ReassemblyError);
            }
            current_pos += bytes.len() as u64;
            buffer.put(bytes);
        }

        if current_pos != content_length {
            return Err(DownloadError::ReassemblyError);
        }

        Ok((Arc::from(buffer.freeze().as_ref()), content_type))
    }
}

#[derive(Debug)]
pub enum DownloadError {
    RequestError(reqwest::Error),
    RequestNotSuccess(u16),
    ContentLengthMissing,
    RangeNotSupported,
    ReassemblyError,
}

impl DownloadError {
    pub fn is_range_not_supported(&self) -> bool {
        match self {
            Self::RangeNotSupported => true,
            _ => false,
        }
    }

    pub fn is_content_length_missing(&self) -> bool {
        match self {
            Self::ContentLengthMissing => true,
            _ => false,
        }
    }
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError(e) => e.fmt(f),
            Self::RequestNotSuccess(status_code) => {
                write!(f, "Server respond with status code {}", status_code)
            }
            Self::ContentLengthMissing => write!(f, "Content-Length header is missing"),
            Self::RangeNotSupported => write!(f, "Server does not support range requests"),
            Self::ReassemblyError => write!(f, "Error reassembling downloaded chunks"),
        }
    }
}

impl Error for DownloadError {}

impl From<reqwest::Error> for DownloadError {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError(value)
    }
}
