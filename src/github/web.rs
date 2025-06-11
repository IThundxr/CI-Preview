use crate::app::App;
use crate::error::Error;
use crate::error::*;
use crate::github::octocrab::models::workflows_extension::{JobsList, WorkflowRun};
use crate::github::verification::GithubEvent;
use crate::util::time::format_duration;
use axum::extract::State;
use octocrab::models::webhook_events::WebhookEventPayload;
use octocrab::models::webhook_events::payload::WorkflowRunWebhookEventAction;
use octocrab::models::workflows::Conclusion;
use regex::Regex;
use serenity::all::colours::branding;
use serenity::all::*;
use snafu::{OptionExt, ResultExt};

const SKIP_PATTERNS: &[&str] = &[
    "[skip ci]",
    "[ci skip]",
    "[no ci]",
    "[skip actions]",
    "[actions skip]",
];

pub async fn handle_github_webhhook(
    State(app): State<App>,
    event: GithubEvent,
) -> Result<&'static str, Error> {
    match event.event.specific {
        WebhookEventPayload::Push(payload) => {
            let message = payload
                .head_commit
                .context(FailedToUnwrapValueSnafu)?
                .message;

            if !SKIP_PATTERNS.iter().any(|skip| message.contains(skip)) {
                app.cache
                    .commits
                    .insert(payload.after, payload.commits)
                    .await
            }
        }
        WebhookEventPayload::WorkflowRun(payload) => {
            let workflow = serde_json::from_value::<WorkflowRun>(payload.workflow_run)
                .context(DeserializationErrorJsonSnafu)?;

            let config = event.repo_config;
            let repo = event.event.repository.context(FailedToUnwrapValueSnafu)?;
            let branch = event.branch;

            // Rethink
            let sender = event.event.sender.context(FailedToUnwrapValueSnafu)?;
            let owner = repo.owner.context(FailedToUnwrapValueSnafu)?;
            let html_url = workflow.inner.html_url;
            let head_sha = workflow.inner.head_sha;
            let run_number = workflow.inner.run_number;

            // TODO - This is incredibly bad
            let mod_version = octocrab::instance()
                .repos_by_id(repo.id)
                .get_content()
                .path(&config.mod_version.path)
                .r#ref(&branch)
                .send()
                .await
                .ok()
                .and_then(|c| c.items.into_iter().next())
                .and_then(|c| octocrab::models::repos::Content::decoded_content(&c))
                .and_then(|c| {
                    let regex = Regex::new(&config.mod_version.regex).ok()?;
                    regex
                        .captures(&c)
                        .and_then(|c| c.get(config.mod_version.group))
                        .map(|c| c.as_str().to_string())
                })
                .context(FailedToUnwrapValueSnafu)?;

            let formatted_mod_version = config
                .mod_version
                .format
                .clone()
                .unwrap_or_else(|| mod_version.to_string())
                .replace("${mod_version}", &mod_version)
                .replace("${minecraft_version}", &config.minecraft_version)
                .replace("${build_number}", &run_number.to_string());

            let commit_info = match app.cache.commits.get(&head_sha).await {
                Some(commits) if !commits.is_empty() => {
                    let mut formatted_commits = commits
                        .iter()
                        .map(|commit| {
                            let commiter = &commit.committer;
                            let username = commiter.username.clone().unwrap();
                            let title = commit
                                .message
                                .split("\n")
                                .next()
                                .unwrap()
                                .replace("#", "\\#");
                            format!(
                                "[➤]({}) {} - [{}](https://github.com/{})",
                                commit.url, title, username, username
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n");

                    // TODO - Magic number :(
                    if formatted_commits.len() > 3072 {
                        formatted_commits = format!(
                            "Commit list is too long to display, please look [here]({html_url}/commits/{head_sha}) instead."
                        )
                    }

                    formatted_commits
                }
                _ => "No commits found".into(),
            };

            if config.workflows.contains(&workflow.path) {
                let embed = |status, extra| {
                    let author = CreateEmbedAuthor::new(format!("{}/{}", repo.name, branch))
                        .icon_url(owner.avatar_url)
                        .url(html_url);

                    let description = format!(
                        r"
                        ## Build <t:{}:R>
                        Status: {}
                        Version: **{}**
                        {}{}",
                        workflow.run_started_at.timestamp(),
                        status,
                        formatted_mod_version,
                        extra,
                        commit_info
                    );

                    let footer = CreateEmbedFooter::new(sender.login).icon_url(sender.avatar_url);

                    CreateEmbed::new()
                        .author(author)
                        .description(description)
                        .footer(footer)
                        .color(branding::BLURPLE)
                };

                match payload.action {
                    WorkflowRunWebhookEventAction::InProgress => {
                        let emoji = app
                            .serenity_http
                            .get_application_emoji(app.emojis.processing)
                            .await
                            .context(FailedToFindEmojiSnafu)?;
                        let status = &format!("Build is running for **#{}** {}", run_number, emoji);

                        let message = event
                            .channel_id
                            .send_message(
                                app.serenity_http,
                                CreateMessage::new().embed(embed(status, "")),
                            )
                            .await
                            .context(FailedToSendMessageSnafu)?;

                        app.cache
                            .running_workflows
                            .insert(workflow.inner.id, message.id)
                            .await;
                    }
                    WorkflowRunWebhookEventAction::Completed => {
                        let message_id = app
                            .cache
                            .running_workflows
                            .get(&workflow.inner.id)
                            .await
                            .context(CannotFindMessageSnafu)?;
                        let mut message = event
                            .channel_id
                            .message(&app.serenity_http, message_id)
                            .await
                            .context(FailedToSendMessageSnafu)?;

                        let time_taken = {
                            let difference = workflow.inner.updated_at - workflow.inner.created_at;
                            format_duration(difference.num_seconds())
                        };

                        let mut edit = EditMessage::new();

                        let mut logs = None;
                        let (run_status, color) = match workflow.conclusion_enum {
                            Some(Conclusion::Success) => {
                                let mut buttons = Vec::new();

                                for (id, button) in config.buttons {
                                    let url = button
                                        .url
                                        .clone()
                                        .map(|i| {
                                            i.replace("${version}", &formatted_mod_version)
                                                .replace("${mod_version}", &mod_version)
                                                .replace(
                                                    "${minecraft_version}",
                                                    &config.minecraft_version,
                                                )
                                                .replace("${build_number}", &run_number.to_string())
                                        })
                                        .unwrap_or_default();

                                    buttons.push(button.convert(&id, &url))
                                }

                                let action_row = CreateActionRow::Buttons(buttons);

                                edit = edit.components(vec![action_row]);

                                let emoji = app
                                    .serenity_http
                                    .get_application_emoji(app.emojis.success)
                                    .await
                                    .context(FailedToFindEmojiSnafu)?;
                                (format!("{emoji} Success"), branding::GREEN)
                            }
                            Some(Conclusion::Failure) => {
                                let jobs_list = app
                                    .https
                                    .get(workflow.inner.jobs_url)
                                    .header(
                                        "User-Agent",
                                        "CI-Preview (https://github.com/IThundxr/CI-Preview)",
                                    )
                                    .send()
                                    .await
                                    .context(ReqwestSnafu)?
                                    .json::<JobsList>()
                                    .await
                                    .context(ReqwestSnafu)?;

                                logs = jobs_list
                                    .jobs
                                    .first()
                                    .map(|i| i.html_url.as_str())
                                    .map(|i| format!("Logs: [Run Logs]({i})\n"));

                                let emoji = app
                                    .serenity_http
                                    .get_application_emoji(app.emojis.failed)
                                    .await
                                    .context(FailedToFindEmojiSnafu)?;
                                (format!("{emoji} Failed"), branding::RED)
                            }
                            _ => ("".into(), branding::BLURPLE),
                        };

                        let status =
                            &format!("**{} #{}** in {}", run_status, run_number, time_taken);

                        let embed = embed(status, &logs.unwrap_or_default()).color(color);
                        edit = edit.embed(embed);

                        let _ = message.edit(app.serenity_http, edit).await;

                        app.cache.running_workflows.remove(&workflow.inner.id).await;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    Ok("Thanks and so long for all fish")
}
