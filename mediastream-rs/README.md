# `mediastream-rs`
A library for parsing and generating m3u8 file.

# Installation
```shell
cargo add mediastream-rs
```
or,  
```toml
mediastream-rs = "*"
```

# Usage
```rust
use mediastream_rs::Parser;
use std::io::Cursor;

// 1. Parse
let mut parser = Parser::new(Cursor::new(r#"
#EXTM3U x-tvg-url="test"
#EXTINF:1 tvg-id="a" provider-type="iptv",A
http://example.com/A.m3u8"#));
parser.parse().unwrap();
let result = parser.get_result();
// Do your works with result...

// 2. Generate
println!("{}", result.to_string());
```

# Why make new wheels?
The existing crates do not meet my needs; they can either only parse m3u8 from a certain path (online or local) or cannot output the parsed m3u8 file back to m3u8.  

# License
MIT
