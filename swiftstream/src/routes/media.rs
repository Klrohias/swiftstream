use std::io::Cursor;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mediastream_rs::format::M3uPlaylist;
use serde::Deserialize;
use url::Url;

use crate::{AppStateRef, internal_error_with_log};

#[derive(Deserialize)]
pub struct MediaRequest {
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
    Query(query): Query<MediaRequest>,
) -> Result<Response, StatusCode> {
    let response = state
        .http_client
        .get(&query.origin)
        .send()
        .await
        .map_err(internal_error_with_log!())?;
    let data = response.bytes().await.map_err(internal_error_with_log!())?;
    state.tracking_pool.track(&query.origin).await;

    // parse
    let mut playlist = tokio::task::spawn_blocking(move || {
        let mut parser = mediastream_rs::Parser::new(Cursor::new(data));
        if let Err(e) = parser.parse() {
            return Err(e);
        }
        Ok(parser.get_result())
    })
    .await
    .map_err(internal_error_with_log!())?
    .map_err(internal_error_with_log!())?;

    prepare_all(&state, &mut playlist, query.origin)
        .await
        .map_err(internal_error_with_log!())?;

    Ok(playlist.to_string().into_response())
}
