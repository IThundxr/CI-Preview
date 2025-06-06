use serde::Deserialize;
use serenity::all::ReactionType;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct RepoSettings {
    pub minecraft_version: String,
    pub mod_loader: String,
    pub variables: HashMap<String, Variable>,
    pub buttons: HashMap<String, Button>,
}

#[derive(Deserialize)]
pub struct Variable {
    pub file: String,
    pub regex: String,
    pub group: u8,
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
