use std::io::Cursor;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use log::warn;
use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;

use crate::{AppStateRef, internal_error_with_log, transfer::parse_m3u8_async};

#[derive(Deserialize)]
pub struct PlaylistQuery {
    pub origin: String,
}
pub async fn get_playlist(
    State(state): State<AppStateRef>,
    Query(query): Query<PlaylistQuery>,
) -> Result<Response, StatusCode> {
    let data = state
        .http_client
        .get(&query.origin)
        .send()
        .await
        .map_err(internal_error_with_log!("Request origin"))?
        .bytes()
        .await
        .map_err(internal_error_with_log!("Request bytes"))?;

    let mut playlist = parse_m3u8_async(Cursor::new(data))
        .await
        .map_err(internal_error_with_log!("Parse m3u8"))?;

    let base = Url::parse(&query.origin).map_err(internal_error_with_log!("Parse url"))?;

    for media in playlist.medias.iter_mut() {
        let media_location = media.location.clone();
        match Url::parse(&media_location) {
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                // RelativeUrlWithoutBase, join with base url
                let joined_url = base
                    .join(&media_location)
                    .map_err(internal_error_with_log!("Join url"))?;

                media.location = format!(
                    "{}/media?origin={}",
                    state.config.base_url,
                    urlencoding::encode(joined_url.as_str())
                )
                .into();
            }
            Err(another_err) => {
                // another error, encode the url directly
                warn!(
                    "Failed to parse url {}, encode directly: {}",
                    media_location, another_err
                );

                media.location = format!(
                    "{}/media?origin={}",
                    state.config.base_url,
                    urlencoding::encode(&media_location)
                )
                .into();
            }
            Ok(v) => {
                media.location = format!(
                    "{}/media?origin={}",
                    state.config.base_url,
                    urlencoding::encode(v.as_str())
                )
                .into();
            }
        };
    }

    Ok(playlist.to_string().into_response())
}
