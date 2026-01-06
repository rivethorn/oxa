use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tiny_http::{Server, Response};
use url::Url;
use oauth2::AuthorizationCode;

use crate::config::SERVER_TIMEOUT_SECS;
use crate::utils::error::AppError;

/// Start local HTTP server to handle OAuth callback
pub async fn start_callback_server(port: u16) -> Result<oneshot::Receiver<AuthorizationCode>, AppError> {
    let (tx, rx) = oneshot::channel::<AuthorizationCode>();
    let csrf_token = oauth2::CsrfToken::new_random();
    let csrf_token_clone = csrf_token.clone();
    
    let server = Arc::new(
        Server::http(format!("127.0.0.1:{}", port))
            .map_err(|e| AppError::ServerError(e.to_string()))?
    );
    
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
                // Handle successful authorization
                if let Some(code) = parsed_url.query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| AuthorizationCode::new(value.into_owned())) 
                {
                    // Verify CSRF token
                    if let Some(returned_state) = parsed_url.query_pairs()
                        .find(|(key, _)| key == "state")
                        .map(|(_, value)| value.into_owned()) 
                    {
                        if returned_state == csrf_token_clone.secret() {
                            let response = Response::from_string(
                                "Authentication successful! You can close this window."
                            ).with_status_code(200);
                            let _ = request.respond(response);
                            let _ = tx.send(Ok(code));
                            break;
                        } else {
                            let response = Response::from_string(
                                "Security error: Invalid state parameter."
                            ).with_status_code(400);
                            let _ = request.respond(response);
                        }
                    }
                } 
                // Handle authorization denial
                else if let Some(error) = parsed_url.query_pairs()
                    .find(|(key, _)| key == "error")
                    .map(|(_, value)| value.into_owned()) 
                {
                    let message = match error.as_str() {
                        "access_denied" => "You denied access to the application.",
                        _ => "Authentication failed.",
                    };
                    
                    let response = Response::from_string(
                        format!("Error: {}", message)
                    ).with_status_code(400);
                    let _ = request.respond(response);
                    let _ = tx.send(Err(AppError::UserCancelled));
                    break;
                }
            }
            
            attempts += 1;
        }
    });
    
    Ok(rx)
}