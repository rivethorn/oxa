use anyhow::Result;
use octocrab::{Octocrab, OctocrabBuilder, models::Repository};
use chrono::{DateTime, Utc};

use crate::utils::error::AppError;

#[derive(Debug, Clone)]
pub struct RepoWithActions {
    pub repo_name: String,  // Use repo name instead of full Repository struct
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
    
    pub async fn get_user_repos(&self) -> Result<Vec<RepoWithActions>, AppError> {
        // For now, return mock data since GitHub API is complex
        // TODO: Implement proper GitHub API integration
        let mut mock_repos = Vec::new();
        
        // Create some mock repositories
        for i in 1..=3 {
            let has_actions = i % 2 == 0;
            let conclusion = match i {
                1 => Some("success"),
                2 => Some("failure"), 
                3 => Some("pending"),
                _ => None,
            };
            
            let last_run = if has_actions {
                Some(WorkflowSummary {
                    id: i as u64,
                    name: format!("workflow_{}.yml", i),
                    status: "completed".to_string(),
                    conclusion: conclusion.map(|s| s.to_string()),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    html_url: format!("https://github.com/{}/actions/runs/{}", self.username, i),
                })
            } else {
                None
            };
            
            // Create mock repository data
            let repo_name = format!("repo-{}", i);
            let has_actions = i % 2 == 0;
            let conclusion = match i {
                1 => Some("success"),
                2 => Some("failure"), 
                3 => Some("pending"),
                _ => None,
            };
            
            let last_run = if has_actions {
                Some(WorkflowSummary {
                    id: i as u64,
                    name: format!("workflow_{}.yml", i),
                    status: "completed".to_string(),
                    conclusion: conclusion.map(|s| s.to_string()),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    html_url: format!("https://github.com/{}/actions/runs/{}", "mock_user", i),
                })
            } else {
                None
            };
            
            mock_repos.push(RepoWithActions {
                repo_name,
                has_actions,
                last_run,
                last_check: chrono::Utc::now(),
            });
        }
        
        Ok(mock_repos)
    }
    
    fn create_mock_repository(_name: &str) -> octocrab::models::Repository {
        // Create a minimal mock repository - this is a placeholder
        // In real implementation, this would come from GitHub API response
        // For now, we need to figure out how to construct Repository properly
        todo!("Proper Repository construction needed")
    }
    
    async fn check_repo_has_workflows(&self, _repo: &Repository) -> Result<bool, AppError> {
        // For now, return mock value
        // TODO: Implement real workflow checking
        Ok(true)
    }
    
    async fn get_last_workflow_run(&self, _repo: &Repository) -> Result<WorkflowSummary, AppError> {
        // For now, return mock workflow run
        // TODO: Implement real workflow run fetching
        Ok(WorkflowSummary {
            id: 1,
            name: "CI/CD".to_string(),
            status: "completed".to_string(),
            conclusion: Some("success".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            html_url: "https://github.com/example/actions".to_string(),
        })
    }
    
    pub async fn get_repo_workflows(&self, _repo: &Repository) -> Result<Vec<WorkflowSummary>, AppError> {
        // For now, return mock workflow data
        // TODO: Implement real workflow fetching
        let mut workflows = Vec::new();
        
        // Create some mock workflow runs
        for i in 1..=3 {
            let conclusion = match i {
                1 => Some("success".to_string()),
                2 => Some("failure".to_string()),
                3 => Some("pending".to_string()),
                _ => None,
            };
            
            workflows.push(WorkflowSummary {
                id: i,
                name: format!("Workflow {}.yml", i),
                status: "completed".to_string(),
                conclusion,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                html_url: format!("https://github.com/example/actions/run/{}", i),
            });
        }
        
        Ok(workflows)
    }
}