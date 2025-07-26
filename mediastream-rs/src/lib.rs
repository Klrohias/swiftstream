//! # mediastream-rs
//! A library for parsing and generating m3u8 file
//!
//! # Example
//! ```rust
//! use mediastream_rs::Parser;
//! use std::io::Cursor;
//!
//! // 1. Parse
//! let mut parser = Parser::new(Cursor::new(r#"
//! #EXTM3U x-tvg-url="test"
//! #EXTINF:1 tvg-id="a" provider-type="iptv",A
//! http://example.com/A.m3u8"#));
//! parser.parse().unwrap();
//! let result = parser.get_result();
//! // Do your works with result...
//!
//! // 2. Generate
//! println!("{}", result.to_string());
//! ```

mod builder;
pub mod format;
mod parser;
pub use parser::*;
