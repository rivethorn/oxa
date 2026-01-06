// OAuth2 Configuration for GitHub
pub const CLIENT_ID: &str = "YOUR_GITHUB_CLIENT_ID";
pub const CLIENT_SECRET: &str = "YOUR_GITHUB_CLIENT_SECRET";

pub const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
pub const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
pub const DEVICE_AUTH_URL: &str = "https://github.com/login/device/code";

pub const SCOPES: &[&str] = &["repo", "read:org", "workflow"];

pub const DEFAULT_PORT: u16 = 8080;
pub const PORT_RANGE: u16 = 100;
pub const SERVER_TIMEOUT_SECS: u64 = 300;

pub const SERVICE_NAME: &str = "oxa";
pub const TOKEN_KEY: &str = "github_oauth_token";

pub const APP_NAME: &str = "OXA - GitHub Actions TUI";
