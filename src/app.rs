use moka::future::Cache;
use octocrab::models::webhook_events::payload::PushWebhookEventCommit;
use octocrab::models::RunId;
use serenity::all::{EmojiId, MessageId};
use serenity::http::Http;
use std::env;
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
        Self {
            https: reqwest::Client::new(),
            serenity_http,
            // TODO - Make TTL configurable
            cache: AppCache {
                commits: Cache::builder()
                    .time_to_live(Duration::from_secs(60 * 60))
                    .build(),
                running_workflows: Cache::builder()
                    .time_to_live(Duration::from_secs(60 * 60))
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
