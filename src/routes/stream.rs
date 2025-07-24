use std::io::{Cursor, Seek, SeekFrom};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

use futures::stream::select_all;
use serde::Deserialize;
use tokio::io::{self, AsyncReadExt};
use tokio_util::io::ReaderStream;

use crate::{
    AppStateRef, bad_request_with_log, internal_error_with_log,
    transfer::{HttpRange, parse_http_ranges},
};

#[derive(Deserialize)]
pub struct StreamRequest {
    pub origin: String,
}

pub async fn get_stream(
    State(state): State<AppStateRef>,
    Query(query): Query<StreamRequest>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let data = state.cache_pool.get(&query.origin).await;
    if data.is_err() {
        if data.as_ref().unwrap_err().kind() == io::ErrorKind::OutOfMemory {
            return Ok(axum::http::Response::builder()
                .header(header::LOCATION, query.origin)
                .status(StatusCode::TEMPORARY_REDIRECT)
                .body(Body::empty())
                .map_err(internal_error_with_log!())?);
        } else {
            data.as_ref().map_err(internal_error_with_log!())?;
        }
    }
    let data = data.unwrap();

    // is it a Range request?
    let ranges = if let Some(range) = headers.get(header::RANGE) {
        let range_str = range.to_str().map_err(internal_error_with_log!())?;
        let ranges = parse_http_ranges(range_str).map_err(bad_request_with_log!())?;
        Some(ranges)
    } else {
        None
    };

    if let Some(ranges) = ranges {
        let body = select_all(
            ranges
                .into_iter()
                .map(|x| (x, Cursor::new(data.bytes.clone())))
                .map(|mut x| match x.0 {
                    HttpRange::Prefix(len) => x.1.take(len),
                    HttpRange::Suffix(len) => {
                        _ = x.1.seek(SeekFrom::End(-(len).try_into().unwrap_or(0)));
                        x.1.take(len)
                    }
                    HttpRange::Range(from, to) => {
                        _ = x.1.seek(SeekFrom::Start(from));
                        x.1.take(to - from)
                    }
                })
                .map(|x| ReaderStream::new(x)),
        );

        let response = Response::builder()
            .header(header::CONTENT_TYPE, data.content_type)
            .body(Body::from_stream(body))
            .map_err(internal_error_with_log!())?;

        return Ok(response);
    } else {
        // send all
        let length = data.bytes.len();
        let body = ReaderStream::new(Cursor::new(data.bytes));

        return Ok((
            [
                (header::CONTENT_TYPE, data.content_type),
                (header::CONTENT_LENGTH, length.to_string()),
            ],
            Body::from_stream(body),
        )
            .into_response());
    }
}
