# SwiftStream 🚀

[English](README.md) | [简体中文](README.zh-CN.md)

## Project Overview
SwiftStream is a high-performance HLS (m3u8) streaming accelerator written in Rust. It acts as a local proxy that caches and prefetches media segments to enable faster playback in local media players.  

## Key Features
- 🚀 Accelerates HLS/m3u8 streaming playback  
- 📦 Local caching of TS segments  
- ⚡ Low-latency proxy server  
- 📊 Configurable cache settings  

## Usage
<details>

<summary>Build and run</summary>

1. Clone and build  

    ```bash
    git clone https://github.com/your-repo/swiftstream-rs.git
    cd swiftstream-rs
    cargo build --release
    ```

2. Configuration  

    See [Configuration](#configuration)  

3. Run  

    ```shell
    ./target/release/swiftstream
    ```

</details>

## Configuration
The program defaults to reading the configuration from `config.yml`. If you need to customize the configuration file path, please use the `SS_CONFIG_PATH` environment variable.  

```yml
# listenAddr, where does your server run at
listenAddr: 0.0.0.0:19198

# baseUrl, where can you access to the server
baseUrl: http://127.0.0.1:19198

# sizeLimit, the maximum RAM size use for caching (in bytes, default: 536870912)
sizeLimit: 536870912 # 512 MB

# cacheExpire, the expire of cached ts segments (in seconds, default: 60)
cacheExpire: 60

# trackExpire, the expire of a media (in seconds, default: 30)
trackExpire: 30

# trackExpire, the interval of starting to prepare a media (in seconds, default: 5)
trackInterval: 5
```
