use std::io::Cursor;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mediastream_rs::format::M3uPlaylist;
use serde::Deserialize;
use url::Url;

use crate::{AppStateRef, internal_error_with_log, transfer::parse_m3u8_async};

#[derive(Deserialize)]
pub struct MediaQuery {
    origin: String,
}

async fn prepare_all(
    state: &AppStateRef,
    playlist: &mut M3uPlaylist,
    origin: impl AsRef<str>,
) -> Result<(), anyhow::Error> {
    let base_url = Url::parse(origin.as_ref())?;

    // prepare all
    for media in playlist.medias.iter_mut() {
        let media_location = media.location.clone();
        let mut location = Url::parse(&media_location);
        if location == Err(url::ParseError::RelativeUrlWithoutBase) {
            location = base_url.join(&media_location);
        }
        let location = location?.to_string();

        state.cache_pool.prepare(&location).await;
        media.location = format!(
            "{}/stream?origin={}",
            state.config.base_url,
            urlencoding::encode(&location)
        )
        .into();
    }

    Ok(())
}

pub async fn get_media(
    State(state): State<AppStateRef>,
    Query(query): Query<MediaQuery>,
) -> Result<Response, StatusCode> {
    let data = state
        .http_client
        .get(&query.origin)
        .send()
        .await
        .map_err(internal_error_with_log!("Request media"))?
        .bytes()
        .await
        .map_err(internal_error_with_log!("Request bytes"))?;

    state.tracking_pool.track(&query.origin).await;

    // parse
    let mut playlist = parse_m3u8_async(Cursor::new(data))
        .await
        .map_err(internal_error_with_log!("Parse m3u8"))?;

    prepare_all(&state, &mut playlist, query.origin)
        .await
        .map_err(internal_error_with_log!("Start caching"))?;

    Ok(playlist.to_string().into_response())
}
