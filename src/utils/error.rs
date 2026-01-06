use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("GitHub API error: {0}")]
    GitHubError(#[from] octocrab::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("JSON error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("No available ports")]
    NoAvailablePorts,

    #[error("Authentication timeout")]
    AuthTimeout,

    #[error("User cancelled authentication")]
    UserCancelled,
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            AppError::AuthFailed(msg) => format!("❌ Authentication failed: {}", msg),
            AppError::NetworkError(e) => {
                if e.is_timeout() {
                    "⏰ Request timed out. Check your internet connection.".to_string()
                } else if e.is_connect() {
                    "🌐 Cannot connect to GitHub. Check your internet connection.".to_string()
                } else {
                    format!("🌐 Network error: {}", e)
                }
            }
            AppError::OAuthError(e) => {
                format!("🔐 OAuth error: {}", e)
            }
            AppError::KeyringError(_) => {
                "🔐 Failed to access secure storage. Please check your system keyring.".to_string()
            }
            AppError::GitHubError(e) => {
                format!("🐙 GitHub API error: {}", e)
            }
            AppError::NoAvailablePorts => {
                "🚫 No available ports for local server. Please try again.".to_string()
            }
            AppError::AuthTimeout => "⏰ Authentication timed out. Please try again.".to_string(),
            AppError::ServerError(msg) => {
                format!("🚫 Server error: {}", msg)
            }
            AppError::UserCancelled => "❌ Authentication was cancelled.".to_string(),
            _ => format!("❌ An error occurred: {}", self),
        }
    }
}
