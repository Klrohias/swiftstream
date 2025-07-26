# SwiftStream 🚀

[English](README.md) | [简体中文](README.zh-CN.md)

## 项目概述
SwiftStream 是一款用 Rust 编写的高性能 HLS (m3u8) 流媒体加速器。它作为本地代理服务器，通过缓存和预加载媒体分段来实现本地播放器的流畅播放加速。

## 核心特性
- 🚀 提升 HLS/m3u8 流媒体播放速度  
- 📦 本地缓存 TS 媒体分段  
- ⚡ 低延迟代理服务  
- 📊 可自定义缓存配置  

## 使用指南
<details>

<summary>自行构建并运行</summary>

1. 克隆并构建  
    ```bash
    git clone https://github.com/your-repo/swiftstream-rs.git
    cd swiftstream-rs
    cargo build --release
    ```

2. 配置  
    参见 [配置说明](#配置说明)  

3. 运行  
    ```shell
    ./target/release/swiftstream
    ```

</details>

## 配置说明
程序默认从 `config.yml` 读取配置。如需指定自定义配置文件路径，请使用 `SS_CONFIG_PATH` 环境变量。

```yml
# listenAddr 服务监听地址
listenAddr: 0.0.0.0:19198

# baseUrl 服务访问地址
baseUrl: http://127.0.0.1:19198

# sizeLimit 内存缓存上限（单位：字节，默认值：536870912）
sizeLimit: 536870912 # 512 MB

# cacheExpire TS分段缓存有效期（单位：秒，默认值：60）
cacheExpire: 60

# trackExpire 媒体资源跟踪有效期（单位：秒，默认值：30）
trackExpire: 30

# trackInterval 媒体预加载间隔（单位：秒，默认值：5）
trackInterval: 5
```
