use serenity::http::Http;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;

pub async fn start() -> Arc<Http> {
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");
    let intents = GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .await
        .expect("Error creating client");

    let http = client.http.clone();

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    http
}
