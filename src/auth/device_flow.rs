use anyhow::Result;
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, 
    RedirectUrl, Scope, TokenUrl,
    basic::BasicClient,
    DeviceAuthorizationUrl, StandardDeviceAuthorizationResponse,
};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::{CLIENT_ID, AUTH_URL, TOKEN_URL, DEVICE_AUTH_URL, SCOPES};
use crate::utils::error::AppError;
use super::auth_code::{AuthResult, get_username};

/// Perform OAuth2 device code flow
pub async fn device_code_flow() -> Result<AuthResult, AppError> {
    let client = BasicClient::new(
        oauth2::ClientId::new(CLIENT_ID.to_string()),
        None, // No client secret needed for device flow
        oauth2::AuthUrl::new(AUTH_URL.to_string())?,
        Some(oauth2::TokenUrl::new(TOKEN_URL.to_string())?),
    )
    .set_device_authorization_url(
        DeviceAuthorizationUrl::new(DEVICE_AUTH_URL.to_string())?
    );

    // Add required scopes
    let mut client = client;
    for scope in SCOPES {
        client = client.add_scope(oauth2::Scope::new(scope.to_string()));
    }

    // Request device authorization
    let device_auth_response: StandardDeviceAuthorizationResponse = client
        .exchange_device_code()
        .request_async(async_http_client()).await?;

    // Display instructions to user
    println!("\n🔐 GitHub Authentication Required");
    println!("Please follow these steps to authenticate:\n");
    println!("1. Visit: {}", device_auth_response.verification_uri());
    println!("2. Enter code: {}\n", device_auth_response.user_code().secret());
    println!("This code will expire in {} minutes.", device_auth_response.expires_in() / 60);
    println!("Waiting for authorization...\n");

    // Poll for token with timeout
    let timeout = Duration::from_secs(device_auth_response.expires_in());
    let token_response = client
        .exchange_device_access_token(&device_auth_response)
        .request_async(async_http_client(), sleep, Some(timeout))
        .await?;

    // Get user information
    let username = get_username(token_response.access_token().secret()).await?;

    Ok(AuthResult {
        access_token: token_response.access_token().secret().clone(),
        username,
    })
}

async fn async_http_client() -> oauth2::reqwest::HttpClient<oauth2::reqwest::Error> {
    oauth2::reqwest::async_http_client()
}