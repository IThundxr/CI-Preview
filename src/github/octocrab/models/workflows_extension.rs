use octocrab::models::workflows::{Conclusion, Job, Run};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WorkflowRun {
    #[serde(flatten)]
    pub inner: Run,
    pub path: String,
    pub run_started_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "conclusion")]
    pub conclusion_enum: Option<Conclusion>,
}

#[derive(Deserialize)]
pub struct JobsList {
    pub jobs: Vec<Job>,
}
