mod bot;
mod config;
mod app;
mod github;

use std::env;
use axum::Router;
use tracing::log::info;
use tracing_subscriber::EnvFilter;
use crate::config::app_config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Config::load_initial();
    // We need to hang onto this for the watcher to actually... watch
    let _ = Config::watch()?;

    let router = Router::new();

    let ip = env::var("APP_IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let address = format!("{ip}:{port}");

    info!("Listening on {address}");

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
