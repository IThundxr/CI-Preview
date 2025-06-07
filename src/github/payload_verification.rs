use crate::config::app_config::Config;
use crate::config::repo_config::RepoSettings;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use hmac_sha256::HMAC;
use octocrab::models::repos::Content;
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use octocrab::models::workflows::Run;
use serenity::all::ChannelId;
use axum::extract::rejection::BytesRejection;
use axum::response::{IntoResponse, Response};
use hex::FromHexError;
use snafu::{OptionExt, ResultExt, Snafu};
use subtle::ConstantTimeEq;
use crate::github::payload_verification::EventError::{DeserializationError, InvalidSignature};

pub struct GithubEvent {
    pub event_type: WebhookEvent,
    pub channel_id: ChannelId,
    pub repo_config: Option<RepoSettings>,
}

#[derive(Snafu, Debug)]
pub enum EventError {
    #[snafu(display("Invalid body"))]
    InvalidBody { source: BytesRejection },
    #[snafu(display("X-GitHub-Event header is invalid"))]
    InvalidHeader,
    #[snafu(display("Encountered error during deserialization"))]
    DeserializationError { source: serde_json::Error },
    #[snafu(display("Repository is missing or invalid"))]
    InvalidRepository,
    #[snafu(display("No config found"))]
    InvalidConfig,
    #[snafu(display("X-Hub-Signature-256 header is missing"))]
    MissingSignatureHeader,
    #[snafu(display("Signature prefix is missing"))]
    MissingSignaturePrefix,
    #[snafu(display("Signature hex is invalid"))]
    InvalidSignatureHex { source: FromHexError },
    #[snafu(display("Invalid Signature"))]
    InvalidSignature,
    #[snafu(display("Unable to get repository config"))]
    FailedToGetRepoConfig,
}

impl IntoResponse for EventError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

impl<S> FromRequest<S> for GithubEvent
where
    S: Send + Sync,
{
    type Rejection = EventError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers().clone();

        let body = Bytes::from_request(req, state)
            .await
            .context(InvalidBodySnafu)?;

        let event = headers
            .get("X-GitHub-Event")
            .and_then(|header| header.to_str().ok())
            .context(InvalidHeaderSnafu)?;

        let event = WebhookEvent::try_from_header_and_body(event, &body)
            .context(DeserializationSnafu)?;

        let event_clone = event.clone();

        let repo = event_clone
            .repository
            .context(InvalidRepositorySnafu)?;

        let config = Config::get()
            .get(repo.url.as_str())
            .cloned()
            .context(InvalidConfigSnafu)?;

        let signature_hash = headers
            .get("X-Hub-Signature-256")
            .and_then(|header| header.to_str().ok())
            .context(MissingSignatureHeaderSnafu)?
            .strip_prefix("sha256=")
            .context(MissingSignaturePrefixSnafu)?;

        let signature = hex::decode(signature_hash)
            .context(InvalidSignatureHexSnafu)?;

        if HMAC::mac(&body, &config.webhook_secret)
            .ct_ne(&signature)
            .into()
        {
            return Err(InvalidSignature);
        }

        let git_ref: Option<String> = match event_clone.specific {
            WebhookEventPayload::Push(payload) => {
                payload.r#ref.strip_prefix("refs/heads/").map(|s| s.into())
            }
            WebhookEventPayload::WorkflowRun(payload) => {
                serde_json::from_value::<Run>(payload.workflow_run)
                    .ok()
                    .map(|r| r.head_branch)
            }
            _ => None,
        };

        let octocrab = octocrab::instance();
        let repo_handler = octocrab.repos_by_id(repo.id);

        let mut builder = repo_handler.get_content().path(".ci-preview.yml");

        if let Some(r#ref) = git_ref {
            builder = builder.r#ref(r#ref);
        }

        let repo_settings = builder
            .send()
            .await
            .ok()
            .and_then(|c| c.items.into_iter().next())
            .and_then(|c| Content::decoded_content(&c))
            .and_then(|c| serde_json::from_str(c.as_str()).ok())
            .context(FailedToGetRepoConfigSnafu)?;

        Ok(GithubEvent {
            event_type: event,
            channel_id: config.channel_id,
            repo_config: repo_settings,
        })
    }
}
