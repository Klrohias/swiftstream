use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use m3u8_rs::Playlist;
use serde::Deserialize;

use crate::{AppStateRef, internal_error_with_log};

#[derive(Deserialize)]
pub struct MediaRequest {
    origin: String,
}

pub async fn get_media(
    State(state): State<AppStateRef>,
    Query(query): Query<MediaRequest>,
) -> Result<Response, StatusCode> {
    let response = state
        .http_client
        .get(query.origin)
        .send()
        .await
        .map_err(internal_error_with_log!())?;
    let data = response.bytes().await.map_err(internal_error_with_log!())?;
    let playlist = m3u8_rs::parse_playlist(&data).map_err(internal_error_with_log!())?;
    match playlist.1 {
        Playlist::MediaPlaylist(media) => {
            // Noop
        }
        Playlist::MasterPlaylist(_) => {
            return Ok((
                StatusCode::NOT_ACCEPTABLE,
                "This is a media playlist".to_string(),
            )
                .into_response());
        }
    }
    Ok(().into_response())
}
