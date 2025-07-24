use axum::{Router, routing::get};

use crate::AppStateRef;

mod media;
mod stream;

pub fn get_routes(app_state: &AppStateRef) -> Router {
    Router::new()
        .route("/stream", get(stream::get_stream))
        .route("/media", get(media::get_media))
        .with_state(app_state.clone())
}
