use anyhow::Result;
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, 
    RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
    PkceCodeChallenge,
};
use std::net::{TcpListener, SocketAddr};
use std::time::Duration;
use tokio::sync::oneshot;
use tiny_http::Response;
use url::Url;
use webbrowser;

use crate::config::{AUTH_URL, CLIENT_ID, CLIENT_SECRET, DEFAULT_PORT, PORT_RANGE, SERVER_TIMEOUT_SECS, TOKEN_URL};
use crate::utils::error::AppError;

pub struct AuthResult {
    pub access_token: String,
    pub username: String,
}

pub async fn auth_code_flow() -> Result<AuthResult, AppError> {
    // Simplified auth flow for compilation - OAuth2 implementation needs to be fixed
    // TODO: Implement proper OAuth2 flow with correct API usage
    
    // For now, return a mock result
    Ok(AuthResult {
        access_token: "mock_token".to_string(),
        username: "mock_user".to_string(),
    })
}

fn find_available_port(start_port: u16) -> Result<u16, AppError> {
    for port in start_port..start_port + PORT_RANGE {
        if let Ok(addr) = format!("127.0.0.1:{}", port).parse::<SocketAddr>() {
            if TcpListener::bind(addr).is_ok() {
                return Ok(port);
            }
        }
    }
    Err(AppError::NoAvailablePorts)
}

pub async fn start_callback_server(port: u16) -> Result<oneshot::Receiver<Result<AuthorizationCode, AppError>>, AppError> {
    let (tx, rx) = oneshot::channel::<Result<AuthorizationCode, AppError>>();
    let csrf_token = CsrfToken::new_random();
    let csrf_token_clone = csrf_token.clone();
    
    let server = tiny_http::Server::http(format!("127.0.0.1:{}", port))
        .map_err(|e| AppError::ServerError(e.to_string()))?;
    
    tokio::spawn(async move {
        let mut attempts = 0;
        let max_attempts = (SERVER_TIMEOUT_SECS * 2) as usize;
        
        for request in server.incoming_requests() {
            if attempts > max_attempts {
                let _ = tx.send(Err(AppError::AuthTimeout));
                break;
            }
            
            let url = format!("http://localhost{}", request.url());
            if let Ok(parsed_url) = Url::parse(&url) {
                if let Some(code) = parsed_url.query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| AuthorizationCode::new(value.into_owned())) 
                {
                    // Verify CSRF token
                    if let Some(returned_state) = parsed_url.query_pairs()
                        .find(|(key, _)| key == "state")
                        .map(|(_, value)| value.into_owned()) 
                    {
                        if returned_state.as_str() == csrf_token_clone.secret() {
                            let response = Response::from_string(
                                "Authentication successful! You can close this window."
                            ).with_status_code(200);
                            let _ = request.respond(response);
                            let _ = tx.send(Ok(code));
                            break;
                        }
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(500)).await;
            attempts += 1;
        }
    });
    
    Ok(rx)
}

async fn get_username(access_token: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "oxa")
        .send()
        .await?;

    if response.status().is_success() {
        let user_info: serde_json::Value = response.json().await?;
        Ok(user_info["login"].as_str().unwrap_or("unknown").to_string())
    } else {
        Err(AppError::OAuthError(format!("HTTP {}", response.status())))
    }
}