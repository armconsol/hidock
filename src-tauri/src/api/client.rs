use anyhow::Result;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{AuthResponse, LoginRequest, UserInfo};

pub struct HiNotesClient {
    base_url: String,
    http_client: Client,
    auth_token: Arc<RwLock<Option<String>>>,
}

impl HiNotesClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: Client::new(),
            auth_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Authenticate with email and password
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<UserInfo> {
        // This will fail initially - TDD RED phase
        todo!("Implement authentication")
    }

    pub async fn get_token(&self) -> Option<String> {
        self.auth_token.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticate_with_credentials() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");

        // Act
        let result = client.authenticate("test@example.com", "password").await;

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert!(!user.id.is_empty());
        assert!(!user.name.is_empty());
    }

    #[tokio::test]
    async fn test_token_is_stored_after_auth() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");

        // Act
        let _ = client.authenticate("test@example.com", "password").await;

        // Assert
        let token = client.get_token().await;
        assert!(token.is_some());
        assert!(!token.unwrap().is_empty());
    }
}
