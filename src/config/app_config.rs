use arc_swap::{ArcSwap, Guard};
use notify::{Error, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::all::ChannelId;
use snafu::{ResultExt, Whatever};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::log::{error, info};

static CONFIG: Lazy<ArcSwap<HashMap<String, Config>>> = Lazy::new(|| {
    let config = Config::load().unwrap_or_else(|e| {
        error!("Failed to load config: {e}");
        Default::default()
    });

    ArcSwap::from_pointee(config)
});

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub repository_url: String,
    pub webhook_secret: String,
    pub channel_id: ChannelId,
}

impl Config {
    fn reload() {
        let result = Self::load().map(|new_config| CONFIG.store(Arc::new(new_config)));

        match result {
            Ok(_) => info!("Configuration reloaded successfully."),
            Err(e) => error!("Failed to reload config: {e}. Keeping existing config."),
        }
    }

    pub fn get() -> Guard<Arc<HashMap<String, Config>>> {
        CONFIG.load()
    }

    fn load() -> Result<HashMap<String, Config>, Whatever> {
        let config_contents =
            fs::read_to_string("./config.yml").whatever_context("Failed to read config.yml")?;
        let parsed = serde_norway::from_str::<HashMap<String, Self>>(&config_contents)
            .whatever_context("Failed to deserialize config.yml")?;

        let mut map = HashMap::new();

        for (_, v) in parsed {
            map.insert(v.repository_url.clone(), v);
        }

        Ok(map)
    }

    pub fn watch() -> Result<INotifyWatcher, Error> {
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, Error>| {
                let event = result.unwrap();

                if event.kind.is_modify() {
                    info!("Received SIGHUP, Reloading config...");
                    Config::reload();
                }
            },
            notify::Config::default(),
        )?;
        watcher.watch(Path::new("config.yml"), RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }
}
