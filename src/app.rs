use moka::future::Cache;
use octocrab::models::webhook_events::payload::PushWebhookEventCommit;
use octocrab::models::RunId;
use serenity::all::{EmojiId, MessageId};
use serenity::http::Http;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct App {
    pub https: reqwest::Client,
    pub serenity_http: Arc<Http>,
    pub cache: AppCache,
    pub emojis: Emojis,
}

#[derive(Clone)]
pub struct AppCache {
    pub commits: Cache<String, Vec<PushWebhookEventCommit>>,
    pub running_workflows: Cache<RunId, MessageId>,
}

#[derive(Clone)]
pub struct Emojis {
    pub processing: EmojiId,
    pub success: EmojiId,
    pub failed: EmojiId,
}

impl App {
    pub fn new(serenity_http: Arc<Http>) -> Self {
        let ttl_time = env::var("CACHE_TTL").ok()
            .and_then(|ttl| ttl.parse::<u64>().ok())
            .unwrap_or_else(|| 60);
        let cache_ttl = Duration::from_secs(ttl_time * 60);
        
        Self {
            https: reqwest::Client::new(),
            serenity_http,
            cache: AppCache {
                commits: Cache::builder()
                    .time_to_live(cache_ttl)
                    .build(),
                running_workflows: Cache::builder()
                    .time_to_live(cache_ttl)
                    .build(),
            },
            emojis: Emojis {
                processing: Emojis::for_env_var("PROCESSING_EMOJI"),
                success: Emojis::for_env_var("SUCCESS_EMOJI"),
                failed: Emojis::for_env_var("FAILED_EMOJI"),
            },
        }
    }
}

impl Emojis {
    fn for_env_var(env_var: &str) -> EmojiId {
        env::var(env_var)
            .expect("Processing emoji not found")
            .parse::<u64>()
            .expect("Failed to parse emoji id")
            .into()
    }
}
