use anyhow::Result;
use keyring::{Entry, Error};
use serde_json;

use crate::config::{SERVICE_NAME, TOKEN_KEY};
use crate::utils::error::AppError;

pub struct TokenManager {
    entry: Entry,
}

impl TokenManager {
    pub fn new() -> Result<Self, AppError> {
        let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
        Ok(Self { entry })
    }

    pub fn store_token(&self, access_token: &str) -> Result<(), AppError> {
        let token_data = TokenData {
            access_token: access_token.to_string(),
        };

        let json = serde_json::to_string(&token_data)?;
        self.entry.set_password(&json)?;
        Ok(())
    }

    pub fn get_token(&self) -> Result<Option<TokenData>, AppError> {
        match self.entry.get_password() {
            Ok(password) => {
                let token_data: TokenData = serde_json::from_str(&password)?;
                Ok(Some(token_data))
            }
            Err(Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::KeyringError(e)),
        }
    }

    pub fn clear_token(&self) -> Result<(), AppError> {
        match self.entry.delete_password() {
            Ok(_) => Ok(()),
            Err(Error::NoEntry) => Ok(()), // Already cleared
            Err(e) => Err(AppError::KeyringError(e)),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenData {
    pub access_token: String,
}
