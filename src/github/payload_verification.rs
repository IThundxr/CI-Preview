use subtle::ConstantTimeEq;
use std::fmt::Display;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use hmac_sha256::HMAC;
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use serenity::all::ChannelId;
use tracing::event;
use tracing::log::error;
use crate::config::app_config::Config;

pub struct GithubEvent {
    pub event_type: WebhookEventType,
    pub channel_id: ChannelId,
}

fn err(err: impl Display) -> (StatusCode, String) {
    error!("{err}");
    (StatusCode::BAD_REQUEST, err.to_string())
}

impl<S> FromRequest<S> for GithubEvent
where
    S: Send + Sync
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers().clone();

        let body = Bytes::from_request(req, state)
            .await
            .map_err(|_| err("Invalid body"))?;

        let event = headers.get("X-GitHub-Event")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| err("X-GitHub-Event header is invalid"))?;

        let event = WebhookEvent::try_from_header_and_body(event, &body)
            .map_err(|_| err("Invalid event"))?;

        let config = event.repository
            .and_then(|r| Config::get().get(r.url.as_str()).cloned())
            .ok_or_else(|| err("No config found"))?;

        let signature_hash = headers.get("X-Hub-Signature-256")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| err("X-Hub-Signature-256 header is missing"))?
            .strip_prefix("sha256=")
            .ok_or_else(|| err("Signature prefix is missing"))?;

        let signature = hex::decode(signature_hash)
            .map_err(|_| err("Signature hex is invalid"))?;

        if HMAC::mac(&body, &config.webhook_secret).ct_ne(&signature).into() {
            return Err(err("Invalid Signature"))
        }

        Ok(GithubEvent {
            event_type: event.kind,
            channel_id: config.channel_id
        })
    }
}