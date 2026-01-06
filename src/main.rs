// GitHub Actions TUI Application
use anyhow::Result;
use std::io;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod app;
mod auth;
mod github;
mod ui;
mod utils;
mod config;

use app::{AppState, AuthState};
use auth::{TokenManager, auth_code_flow};
use github::GitHubClient;
use utils::error::AppError;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize application state
    let mut app = AppState::new();
    let token_manager = TokenManager::new()?;

    // Mock data for testing - disabled for now
    // app.repos.push(create_mock_repo("oxa", true, Some("success")));
    // app.repos.push(create_mock_repo("test-repo", true, Some("failure")));
    // app.repos.push(create_mock_repo("no-actions", false, None));
    // app.auth_state = AuthState::Authenticated { username: "testuser".to_string() };
    app.auth_state = AuthState::Unauthenticated;

    // Main application loop
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

// Mock repository function commented out due to Repository struct construction issues
// In a real implementation, this would fetch data from GitHub API
// Mock repository creation commented out due to Repository struct construction issues
// In real implementation, this would fetch data from GitHub API
// fn create_mock_repo(name: &str, has_actions: bool, conclusion: Option<&str>) -> crate::github::RepoWithActions {
//     todo!("Implement proper repository mocking")
// }

async fn load_repositories(app: &mut AppState) -> Result<(), AppError> {
    if let Ok(token_manager) = TokenManager::new() {
        if let Ok(Some(token_data)) = token_manager.get_token() {
            // Load repositories using stored token
            app.set_loading(true);
            
            match GitHubClient::new(&token_data.access_token).await {
                Ok(client) => {
                    match client.get_user_repos().await {
                        Ok(repos) => {
                            app.repos = repos;
                        }
                        Err(e) => {
                            app.auth_state = AuthState::Error { 
                                message: format!("Failed to load repositories: {}", e) 
                            };
                        }
                    }
                }
                Err(e) => {
                    app.auth_state = AuthState::Error { 
                        message: format!("Failed to create GitHub client: {}", e) 
                    };
                }
            }
            
            app.set_loading(false);
        }
    }
    Ok(())
}

async fn load_workflow_actions(app: &mut AppState) -> Result<(), AppError> {
    if let Some(selected_repo_index) = app.selected_repo {
        if selected_repo_index < app.repos.len() {
            let selected_repo = app.repos[selected_repo_index].clone();
            
            if let Ok(token_manager) = TokenManager::new() {
                if let Ok(Some(token_data)) = token_manager.get_token() {
                    match GitHubClient::new(&token_data.access_token).await {
                        Ok(_client) => {
                            // For now, use mock workflow actions
                            app.set_loading(true);
                            app.actions.clear();
                            app.set_loading(false);
                        }
                        Err(_) => {
                            // Error loading workflows
                            app.actions.clear();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(f, app))?;

        // Handle events
        if let Ok(Event::Key(key)) = crossterm::event::read() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let old_selected = app.selected_repo;
                    app.move_selection(-1);
                    
                    // Load workflow actions if repository selection changed
                    if old_selected != app.selected_repo {
                        let _ = load_workflow_actions(app).await;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let old_selected = app.selected_repo;
                    app.move_selection(1);
                    
                    // Load workflow actions if repository selection changed
                    if old_selected != app.selected_repo {
                        let _ = load_workflow_actions(app).await;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    app.switch_panel(-1);
                }
                KeyCode::Right => {
                    app.switch_panel(1);
                }
                KeyCode::Tab => {
                    app.switch_panel(1);
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    // Handle login/logout based on current auth state
                    match &app.auth_state {
                        app::AuthState::Unauthenticated => {
                            // Set to authenticating state for UI feedback
                            app.auth_state = app::AuthState::Authenticating;
                            
                            // Attempt real authentication
                            match auth_code_flow().await {
                                Ok(auth_result) => {
                                    // Store token
                                    let _token_manager = TokenManager::new().and_then(|tm| {
                                        tm.store_token(&auth_result.access_token)
                                    });
                                    
                                    // Update auth state with real username
                                    app.auth_state = app::AuthState::Authenticated { 
                                        username: auth_result.username 
                                    };
                                    
                                    // Clear previous data and load repositories
                                    app.repos.clear();
                                    app.actions.clear();
                                    app.selected_repo = None;
                                    app.selected_action = None;
                                    
                                    // Load user repositories
                                    if let Err(e) = load_repositories(app).await {
                                        app.auth_state = AuthState::Error { 
                                            message: format!("Failed to load repositories: {}", e) 
                                        };
                                    }
                                }
                                Err(e) => {
                                    app.auth_state = app::AuthState::Error { 
                                        message: format!("Login failed: {}", e) 
                                    };
                                }
                            }
                        }
                        app::AuthState::Authenticated { .. } => {
                            // Logout
                            app.auth_state = app::AuthState::Unauthenticated;
                            app.repos.clear();
                            app.actions.clear();
                            app.selected_repo = None;
                            app.selected_action = None;
                            
                            // Clear stored token
                            if let Ok(token_manager) = TokenManager::new() {
                                let _ = token_manager.clear_token();
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}