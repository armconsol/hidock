use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE_NAME: &str = "com.hidock.hinotes.desktop";
const TOKEN_KEY: &str = "oauth_tokens";

/// OAuth2 token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at: u64,
    pub scope: Option<String>,
}

impl TokenData {
    /// Create a new TokenData instance
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        token_type: String,
        expires_in: u64,
        scope: Option<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            access_token,
            refresh_token,
            token_type,
            expires_in,
            expires_at: now + expires_in,
            scope,
        }
    }

    /// Check if the access token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        // Consider expired if less than 5 minutes remaining
        now + 300 >= self.expires_at
    }

    /// Get seconds until expiration
    pub fn seconds_until_expiry(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        self.expires_at as i64 - now as i64
    }
}

/// Secure token storage using system keychain
pub struct TokenStorage {
    entry: Entry,
}

impl TokenStorage {
    /// Create a new TokenStorage instance
    pub fn new(username: &str) -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, username).context("Failed to create keyring entry")?;

        Ok(Self { entry })
    }

    /// Store tokens securely in the system keychain
    pub fn store_tokens(&self, tokens: &TokenData) -> Result<()> {
        let json = serde_json::to_string(tokens).context("Failed to serialize token data")?;

        self.entry
            .set_password(&json)
            .context("Failed to store tokens in keychain")?;

        log::info!("Tokens stored securely in system keychain");
        Ok(())
    }

    /// Retrieve tokens from the system keychain
    pub fn retrieve_tokens(&self) -> Result<TokenData> {
        let json = self
            .entry
            .get_password()
            .context("Failed to retrieve tokens from keychain")?;

        let tokens: TokenData =
            serde_json::from_str(&json).context("Failed to deserialize token data")?;

        Ok(tokens)
    }

    /// Delete tokens from the system keychain
    pub fn delete_tokens(&self) -> Result<()> {
        self.entry
            .delete_credential()
            .context("Failed to delete tokens from keychain")?;

        log::info!("Tokens deleted from system keychain");
        Ok(())
    }

    /// Check if tokens exist in storage
    pub fn has_tokens(&self) -> bool {
        self.entry.get_password().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires keyring/system integration
    fn test_token_expiry_detection() {
        // Token that expires in 10 seconds
        let tokens = TokenData::new(
            "test_token".to_string(),
            Some("refresh".to_string()),
            "Bearer".to_string(),
            10,
            None,
        );

        // Should not be expired yet (10 seconds > 5 minute threshold)
        assert!(!tokens.is_expired());

        // Token that expires in 1 second
        let tokens = TokenData::new(
            "test_token".to_string(),
            Some("refresh".to_string()),
            "Bearer".to_string(),
            1,
            None,
        );

        // Should be considered expired (1 second < 5 minute threshold)
        assert!(tokens.is_expired());
    }

    #[test]
    fn test_token_serialization() {
        let tokens = TokenData::new(
            "access123".to_string(),
            Some("refresh456".to_string()),
            "Bearer".to_string(),
            3600,
            Some("read write".to_string()),
        );

        let json = serde_json::to_string(&tokens).unwrap();
        let deserialized: TokenData = serde_json::from_str(&json).unwrap();

        assert_eq!(tokens.access_token, deserialized.access_token);
        assert_eq!(tokens.refresh_token, deserialized.refresh_token);
        assert_eq!(tokens.token_type, deserialized.token_type);
    }
}
