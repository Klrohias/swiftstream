use axum::{Router, routing::get};

use crate::AppStateRef;

mod media;
mod playlist;
mod stream;

pub fn get_routes(app_state: &AppStateRef) -> Router {
    Router::new()
        .route("/playlist", get(playlist::get_playlist))
        .route(
            "/stream",
            get(stream::get_stream).head(stream::get_stream_head),
        )
        .route("/media", get(media::get_media))
        .with_state(app_state.clone())
}
