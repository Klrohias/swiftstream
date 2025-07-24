use std::{env, sync::Arc};

use anyhow::Result;
use axum::Router;
use swiftstream_rs::{AppState, AppStateRef, load_config, routes};
use tokio::net::TcpListener;

async fn build_app(app_state: &AppStateRef) -> Result<Router> {
    let root = Router::new().merge(routes::get_routes(app_state));

    Ok(root)
}

async fn app_entry() -> Result<()> {
    let config = load_config(env::var("SS_CONFIG_PATH").unwrap_or_else(|_| "config.yml".into()))?;
    let app_state = Arc::new(AppState::new(config));

    let tcp_listener = TcpListener::bind(&app_state.config.listen_addr).await?;
    axum::serve(tcp_listener, build_app(&app_state).await?).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    if let Err(e) = app_entry().await {
        panic!("Fatal error: {}", e);
    }
}
