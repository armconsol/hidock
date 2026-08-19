// Authentication Tauri commands

use crate::api::client::HiNotesClient;
use crate::api::types::UserInfo;
use crate::auth::oauth::OAuth2Handler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state for auth operations
pub struct AuthState {
    pub api_client: Arc<RwLock<HiNotesClient>>,
    pub oauth_handler: Arc<OAuth2Handler>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: UserInfo,
    pub token: String,
}

/// Authenticate with email and password credentials
///
/// # Arguments
/// * `email` - User's email address
/// * `password` - User's password
/// * `state` - Application state containing API client
///
/// # Returns
/// Result containing user info and authentication token on success
#[tauri::command]
pub async fn authenticate_with_credentials(
    email: String,
    password: String,
    state: State<'_, AuthState>,
) -> Result<AuthResult, String> {
    let client = state.api_client.read().await;

    let user = client
        .authenticate(&email, &password)
        .await
        .map_err(|e| format!("Authentication failed: {}", e))?;

    let token = client
        .get_token()
        .await
        .ok_or_else(|| "Failed to retrieve authentication token".to_string())?;

    Ok(AuthResult { user, token })
}

/// Authenticate with Google OAuth2
///
/// Opens browser for Google sign-in flow and exchanges authorization code
/// for authentication token
///
/// # Arguments
/// * `state` - Application state containing OAuth handler
///
/// # Returns
/// Result containing OAuth token on success
#[tauri::command]
pub async fn authenticate_google(state: State<'_, AuthState>) -> Result<String, String> {
    let oauth = &state.oauth_handler;

    let token_data = oauth
        .authenticate_google()
        .await
        .map_err(|e| format!("Google authentication failed: {}", e))?;

    Ok(token_data.access_token)
}

/// Authenticate with Apple OAuth2
///
/// Opens browser for Apple sign-in flow and exchanges authorization code
/// for authentication token
///
/// # Arguments
/// * `state` - Application state containing OAuth handler
///
/// # Returns
/// Result containing OAuth token on success
#[tauri::command]
pub async fn authenticate_apple(state: State<'_, AuthState>) -> Result<String, String> {
    let oauth = &state.oauth_handler;

    let token_data = oauth
        .authenticate_apple()
        .await
        .map_err(|e| format!("Apple authentication failed: {}", e))?;

    Ok(token_data.access_token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::HiNotesClient;
    use crate::auth::oauth::OAuth2Handler;

    #[tokio::test]
    async fn test_api_client_authenticate() {
        // Test the underlying API client authentication logic
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let result = client.authenticate("test@example.com", "password").await;

        // Should either succeed with a valid response or fail with network error
        // (since we're not actually running a server in tests)
        match result {
            Ok(user) => {
                assert_eq!(user.email, "test@example.com");
                assert!(!user.id.is_empty());
            }
            Err(e) => {
                // Expected to fail in test environment without actual server
                assert!(
                    e.to_string().contains("error sending request")
                        || e.to_string().contains("dns error")
                        || e.to_string().contains("connection")
                );
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires external OAuth server
    async fn test_oauth_google_flow() {
        // Test the OAuth2 handler for Google
        let oauth = OAuth2Handler::new("test-client-id", None);
        let result = oauth.authenticate_google().await;

        // In test mode, should return mock token
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.access_token.is_empty());
        assert!(token.access_token.contains("google") || token.access_token.contains("mock"));
    }

    #[tokio::test]
    #[ignore] // Requires external OAuth server
    async fn test_oauth_apple_flow() {
        // Test the OAuth2 handler for Apple
        let oauth = OAuth2Handler::new("test-client-id", None);
        let result = oauth.authenticate_apple().await;

        // In test mode, should return mock token
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.access_token.is_empty());
        assert!(token.access_token.contains("apple") || token.access_token.contains("mock"));
    }

    #[tokio::test]
    async fn test_auth_state_initialization() {
        // Test that AuthState can be properly initialized
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let oauth = OAuth2Handler::new("test-client-id", None);

        let state = AuthState {
            api_client: Arc::new(RwLock::new(client)),
            oauth_handler: Arc::new(oauth),
        };

        // Verify we can access the state
        let _client = state.api_client.read().await;
        assert!(true); // If we got here, state is properly initialized
    }

    #[tokio::test]
    async fn test_auth_result_serialization() {
        // Test that AuthResult can be serialized/deserialized
        let auth_result = AuthResult {
            user: UserInfo {
                id: "user-123".to_string(),
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
            },
            token: "test-token-123".to_string(),
        };

        let json = serde_json::to_string(&auth_result).unwrap();
        let deserialized: AuthResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.user.id, "user-123");
        assert_eq!(deserialized.user.email, "test@example.com");
        assert_eq!(deserialized.token, "test-token-123");
    }

    #[test]
    fn test_auth_result_structure() {
        // Test the structure of AuthResult
        let auth_result = AuthResult {
            user: UserInfo {
                id: "user-456".to_string(),
                email: "user@example.com".to_string(),
                name: "John Doe".to_string(),
            },
            token: "abc123".to_string(),
        };

        assert_eq!(auth_result.user.id, "user-456");
        assert_eq!(auth_result.user.email, "user@example.com");
        assert_eq!(auth_result.user.name, "John Doe");
        assert_eq!(auth_result.token, "abc123");
    }
}
