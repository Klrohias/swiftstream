use std::{error::Error, fmt::Display, str::FromStr};

pub enum HttpRange {
    Range(u64, u64),
    Suffix(u64),
    Prefix(u64),
}

#[derive(Debug)]
pub enum HttpRangeParseError {
    InvalidHeader,
    InvalidRange,
    InvalidNumber(<u64 as FromStr>::Err),
}

impl Display for HttpRangeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHeader => write!(f, "Invalid header"),
            Self::InvalidRange => write!(f, "Invalid range"),
            Self::InvalidNumber(e) => e.fmt(f),
        }
    }
}

impl Error for HttpRangeParseError {}

pub fn parse_http_ranges(value: impl AsRef<str>) -> Result<Vec<HttpRange>, HttpRangeParseError> {
    let value = value.as_ref().trim();
    if !value.starts_with("bytes=") {
        return Err(HttpRangeParseError::InvalidHeader);
    }

    let mut result = Vec::new();
    for range_vec in value
        .split(',')
        .map(|x| x.trim())
        .map(|x| x.split('-').collect::<Vec<_>>())
    {
        if range_vec.len() != 2 {
            return Err(HttpRangeParseError::InvalidRange);
        }

        let from = range_vec[0];
        let to = range_vec[1];
        if from.len() == 0 {
            // suffix-length
            result.push(HttpRange::Suffix(
                str::parse(to).map_err(|e| HttpRangeParseError::InvalidNumber(e))?,
            ));
            break; // only 1 suffix-length
        }

        if to.len() == 0 {
            // range-start
            result.push(HttpRange::Prefix(
                str::parse(from).map_err(|e| HttpRangeParseError::InvalidNumber(e))?,
            ));
            break; // only 1 range-start
        }

        result.push(HttpRange::Range(
            str::parse(from).map_err(|e| HttpRangeParseError::InvalidNumber(e))?,
            str::parse(to).map_err(|e| HttpRangeParseError::InvalidNumber(e))?,
        ));
    }

    Ok(result)
}
