use anyhow::Result;
use octocrab::{Octocrab, OctocrabBuilder, models::Repository};
use chrono::{DateTime, Utc};

use crate::utils::error::AppError;

#[derive(Debug, Clone)]
pub struct RepoWithActions {
    pub repo: Repository,
    pub has_actions: bool,
    pub last_run: Option<WorkflowSummary>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WorkflowSummary {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub html_url: String,
}

pub struct GitHubClient {
    client: Octocrab,
    pub username: String,
}

impl GitHubClient {
    pub async fn new(access_token: &str) -> Result<Self, AppError> {
        let client = OctocrabBuilder::new()
            .personal_token(access_token.to_string())
            .build()?;
        
        let user = client.current().user().await?;
        
        Ok(Self {
            client,
            username: user.login,
        })
    }
}