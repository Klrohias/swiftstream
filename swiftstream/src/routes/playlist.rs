use std::io::Cursor;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
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
        .map_err(internal_error_with_log!())?
        .bytes()
        .await
        .map_err(internal_error_with_log!())?;

    let mut playlist = parse_m3u8_async(Cursor::new(data))
        .await
        .map_err(internal_error_with_log!())?;

    let base = Url::parse(&query.origin).map_err(internal_error_with_log!())?;

    for media in playlist.medias.iter_mut() {
        let media_location = media.location.clone();
        let mut location = Url::parse(&media_location);
        if location == Err(url::ParseError::RelativeUrlWithoutBase) {
            location = base.join(&media_location);
        }
        let location = location.map_err(internal_error_with_log!())?.to_string();

        media.location = format!(
            "{}/media?origin={}",
            state.config.base_url,
            urlencoding::encode(&location)
        )
        .into();
    }

    Ok(playlist.to_string().into_response())
}
