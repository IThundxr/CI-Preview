mod app;
mod bot;
mod config;
mod error;
mod github;
mod util;

use crate::app::App;
use crate::config::app_config::Config;
use crate::github::web::handle_github_webhhook;
use axum::Router;
use axum::routing::post;
use snafu::{ResultExt, Whatever};
use std::env;
use tracing::log::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Whatever> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Use an authenticated instance if a token is passed
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if let Ok(instance) = octocrab::OctocrabBuilder::new()
            .personal_token(token)
            .build()
        {
            octocrab::initialise(instance);
        }
    }

    // Start the bot client
    let bot_http = bot::start().await;

    // We need to hang onto this for the watcher to actually... watch
    let _ = Config::watch();

    let router = Router::new()
        .route("/github/webhook", post(handle_github_webhhook))
        .with_state(App::new(bot_http));

    let ip = env::var("APP_IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let address = format!("{ip}:{port}");

    info!("Listening on {address}");

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_whatever_context(|_| format!("Failed to bind listener to {address}"))?;
    axum::serve(listener, router)
        .await
        .with_whatever_context(|_| format!("Failed to serve listener to {address}"))?;

    Ok(())
}
