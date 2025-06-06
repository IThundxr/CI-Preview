use std::collections::HashMap;
use tracing::log::{error, info};
use notify::{Error, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::fs;
use std::iter::Map;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use anyhow::anyhow;
use arc_swap::{ArcSwap, Guard};
use serenity::all::ChannelId;

static CONFIG: OnceLock<ArcSwap<HashMap<String, Config>>> = OnceLock::new();

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub repository_url: String,
    pub webhook_secret: String,
    pub channel_id: ChannelId,
}

impl Config {
    fn reload() {
        let result = Self::load()
            .map(|new_config| {
                CONFIG.get()
                    .unwrap()
                    .store(Arc::new(new_config))
            });

        match result {
            Ok(_) => info!("Configuration reloaded successfully."),
            Err(e) => error!("Failed to reload config: {e}. Keeping existing config."),
        }
    }

    pub fn get() -> Guard<Arc<HashMap<String, Config>>> {
        CONFIG.get()
            .unwrap()
            .load()
    }
    
    pub fn get_arc() -> Arc<HashMap<String, Config>> {
        CONFIG.get()
            .unwrap()
            .load_full()
    }

    fn load() -> anyhow::Result<HashMap<String, Config>> {
        let config_contents = fs::read_to_string("./config.yml")?;
        let parsed = serde_norway::from_str::<HashMap<String, Self>>(&config_contents)
            .map_err(|e| anyhow!(e))?;

        let mut map = HashMap::new();

        for (k, v) in parsed {
            map.insert(v.repository_url.clone(), v);
        }

        Ok(map)
    }
    
    pub fn load_initial() {
        let config = Self::load().unwrap_or_else(|e| {
            error!("Failed to load config: {e}");
            Default::default()
        });
        
        CONFIG.set(ArcSwap::from_pointee(config))
            .expect("Config already loaded.");
    }

    pub fn watch() -> anyhow::Result<INotifyWatcher> {
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