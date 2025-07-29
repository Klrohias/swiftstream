use std::{fmt::Display, io::BufRead};

use mediastream_rs::{ParseError, format::M3uPlaylist};
use std::error::Error;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum ParseM3U8Error {
    ParseError(ParseError),
    JoinError(JoinError),
}

impl Display for ParseM3U8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JoinError(e) => e.fmt(f),
            Self::ParseError(e) => e.fmt(f),
        }
    }
}

impl Error for ParseM3U8Error {}

impl From<JoinError> for ParseM3U8Error {
    fn from(value: JoinError) -> Self {
        Self::JoinError(value)
    }
}

impl From<ParseError> for ParseM3U8Error {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

pub async fn parse_m3u8_async(
    stream: impl BufRead + Send + 'static,
) -> Result<M3uPlaylist, ParseM3U8Error> {
    Ok(tokio::task::spawn_blocking(move || {
        let mut parser = mediastream_rs::Parser::new(stream);
        if let Err(e) = parser.parse() {
            return Err(e);
        }
        Ok(parser.get_playlist())
    })
    .await??)
}
