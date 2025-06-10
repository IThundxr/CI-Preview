use serde::Deserialize;
use serenity::all::{CreateButton, ReactionType};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct RepoSettings {
    pub minecraft_version: String,
    pub workflows: Vec<String>,
    pub mod_version: ModVersionVariable,
    #[serde(default)]
    pub buttons: HashMap<String, Button>,
}

#[derive(Deserialize)]
pub struct ModVersionVariable {
    pub path: String,
    pub regex: String,
    pub group: usize,
    pub format: Option<String>,
}

#[derive(Deserialize)]
pub struct Button {
    pub style: ButtonStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ReactionType>,
    #[serde(default, skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub disabled: bool,
}

impl Button {
    pub fn convert(&self, id: &str, url: &str) -> CreateButton {
        let mut button: CreateButton = match &self.style {
            ButtonStyle::Link => CreateButton::new_link(url),
            _ => CreateButton::new(id),
        }
        .disabled(self.disabled);

        if let Some(label) = &self.label {
            button = button.label(label);
        }

        if let Some(emoji) = &self.emoji {
            button = button.emoji(emoji.clone());
        }

        button
    }
}

#[derive(Deserialize)]
pub enum ButtonStyle {
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "secondary")]
    Secondary,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "danger")]
    Danger,
    #[serde(rename = "link")]
    Link,
}
