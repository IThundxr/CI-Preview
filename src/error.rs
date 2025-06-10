use axum::extract::rejection::BytesRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hex::FromHexError;
use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    // Deserialization
    #[snafu(display("Encountered error during json deserialization"))]
    DeserializationErrorJson { source: serde_json::Error },

    // Unsorted
    #[snafu(display("Invalid body"))]
    InvalidBody { source: BytesRejection },
    #[snafu(display("X-GitHub-Event header is invalid"))]
    InvalidHeader,
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
    #[snafu(display("Failed to send message"))]
    FailedToSendMessage { source: serenity::Error },
    #[snafu(display("Failed to unwrap value"))]
    FailedToUnwrapValue,
    #[snafu(display("Invalid Regex"))]
    InvalidRegex { source: regex::Error },
    #[snafu(display("Cannot find message"))]
    CannotFindMessage,
    #[snafu(display("Failed to send http request: {}", source))]
    Reqwest { source: reqwest::Error },
    #[snafu(display("Failed to find application emoji"))]
    FailedToFindEmoji { source: serenity::Error },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
