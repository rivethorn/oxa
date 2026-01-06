// Simplified main for initial compilation
use anyhow::Result;
use std::io;
use url::Url;
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
use auth::TokenManager;

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
                    app.move_selection(-1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.move_selection(1);
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
                            
                            // Simulate login delay (optional)
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            
                            // For now, simulate successful login
                            app.auth_state = app::AuthState::Authenticated { 
                                username: "demo_user".to_string() 
                            };
                        }
                        app::AuthState::Authenticated { .. } => {
                            // Logout
                            app.auth_state = app::AuthState::Unauthenticated;
                            app.repos.clear();
                            app.actions.clear();
                            app.selected_repo = None;
                            app.selected_action = None;
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