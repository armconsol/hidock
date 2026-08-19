use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Response, Server};
use tokio::sync::oneshot;
use url::Url;

use super::token_storage::{TokenData, TokenStorage};

const HINOTES_API_BASE: &str = "https://hinotes.hidock.com/v1";
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const APPLE_AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const APPLE_TOKEN_URL: &str = "https://appleid.apple.com/auth/token";
const AUTH_TIMEOUT_SECS: u64 = 300; // 5 minutes
const TOKEN_EXCHANGE_TIMEOUT_SECS: u64 = 30;

/// OAuth2 error types
#[derive(Debug, thiserror::Error)]
pub enum OAuth2Error {
    #[error("Authentication timeout after {0} seconds")]
    Timeout(u64),

    #[error("User cancelled authentication")]
    UserCancelled,

    #[error("Invalid authorization code received")]
    InvalidAuthCode,

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Invalid redirect callback: {0}")]
    InvalidCallback(String),
}

/// OAuth2 token response from Google
#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
}

/// OAuth2 token response from Apple
#[derive(Debug, Deserialize)]
struct AppleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: u64,
    id_token: String, // JWT containing user information
}

/// Decoded Apple ID token claims
#[derive(Debug, Deserialize)]
struct AppleIdTokenClaims {
    iss: String,        // Issuer (https://appleid.apple.com)
    sub: String,        // Subject (unique user identifier)
    aud: String,        // Audience (your client ID)
    iat: u64,           // Issued at timestamp
    exp: u64,           // Expiration timestamp
    email: Option<String>, // User's email (only on first auth)
    email_verified: Option<bool>,
    is_private_email: Option<bool>, // True if using Apple's email relay
    real_user_status: Option<u8>, // 0=unsupported, 1=unknown, 2=likely real
}

/// Apple user information from first-time authorization
#[derive(Debug, Deserialize)]
struct AppleUserInfo {
    name: Option<AppleUserName>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppleUserName {
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
}

/// OAuth2 handler for Google and Apple sign-in
pub struct OAuth2Handler {
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    http_client: Client,
    /// Apple-specific team ID (required for client secret generation)
    apple_team_id: Option<String>,
    /// Apple-specific key ID (required for client secret generation)
    apple_key_id: Option<String>,
}

impl OAuth2Handler {
    pub fn new(client_id: &str, client_secret: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(TOKEN_EXCHANGE_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client_id: client_id.to_string(),
            client_secret,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            http_client,
            apple_team_id: None,
            apple_key_id: None,
        }
    }

    /// Create OAuth2Handler with Apple-specific configuration
    pub fn new_for_apple(
        client_id: &str,
        team_id: &str,
        key_id: &str,
        client_secret: Option<String>,
    ) -> Self {
        let mut handler = Self::new(client_id, client_secret);
        handler.apple_team_id = Some(team_id.to_string());
        handler.apple_key_id = Some(key_id.to_string());
        handler
    }

    /// Generate PKCE code verifier and challenge
    fn generate_pkce() -> (String, String) {
        let mut rng = rand::thread_rng();
        let code_verifier: String = (0..128)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                    .chars()
                    .nth(idx)
                    .unwrap()
            })
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        (code_verifier, code_challenge)
    }

    /// Build Google OAuth2 authorization URL
    fn build_google_auth_url(&self, code_challenge: &str, state: &str) -> String {
        let mut url = Url::parse(GOOGLE_AUTH_URL).expect("Invalid Google auth URL");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", state)
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");

        url.to_string()
    }

    /// Build Apple OAuth2 authorization URL
    fn build_apple_auth_url(&self, state: &str) -> String {
        let mut url = Url::parse(APPLE_AUTH_URL).expect("Invalid Apple auth URL");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("response_mode", "form_post") // Apple requires form_post for web apps
            .append_pair("scope", "name email")
            .append_pair("state", state);

        url.to_string()
    }

    /// Start local HTTP server to receive OAuth callback
    /// Handles both GET (Google) and POST (Apple form_post) requests
    async fn start_callback_server(
        expected_state: String,
    ) -> Result<(String, oneshot::Receiver<Result<String, OAuth2Error>>), OAuth2Error> {
        let server = Server::http("127.0.0.1:8080")
            .map_err(|e| OAuth2Error::ServerError(format!("Failed to start HTTP server: {}", e)))?;

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            if let Ok(mut request) = server.recv() {
                let method = request.method().as_str();
                let url_str = format!("http://localhost{}", request.url());
                log::debug!("Received OAuth callback: {} {}", method, url_str);

                let result = {
                    // Parse parameters from either query string (GET) or body (POST)
                    let params: HashMap<String, String> = if method == "POST" {
                        // Parse POST body for Apple's form_post response mode
                        let mut body = String::new();
                        {
                            let mut reader = request.as_reader();
                            let _ = reader.read_to_string(&mut body);
                        }

                        log::debug!("POST body: {}", body);

                        // Parse form-encoded data
                        form_urlencoded::parse(body.as_bytes())
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect()
                    } else {
                        // Parse query parameters for GET requests (Google)
                        match Url::parse(&url_str) {
                            Ok(url) => url
                                .query_pairs()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect(),
                            Err(e) => {
                                log::error!("Failed to parse callback URL: {}", e);
                                HashMap::new()
                            }
                        }
                    };

                    // Send success HTML response
                    let html = r#"
                        <!DOCTYPE html>
                        <html>
                        <head><title>Authentication Successful</title></head>
                        <body style="font-family: Arial; text-align: center; padding: 50px;">
                            <h1>✓ Authentication Successful</h1>
                            <p>You can close this window and return to the application.</p>
                        </body>
                        </html>
                    "#;
                    let _ = request.respond(Response::from_string(html).with_status_code(200));

                    // Validate state parameter
                    if let Some(state) = params.get("state") {
                        if state != &expected_state {
                            Err(OAuth2Error::InvalidCallback(
                                "State parameter mismatch".to_string(),
                            ))
                        } else if let Some(error) = params.get("error") {
                            // Check for error parameter
                            if error == "access_denied" {
                                Err(OAuth2Error::UserCancelled)
                            } else {
                                Err(OAuth2Error::InvalidCallback(format!(
                                    "OAuth error: {}",
                                    error
                                )))
                            }
                        } else if let Some(code) = params.get("code") {
                            // Extract authorization code
                            Ok(code.clone())
                        } else {
                            Err(OAuth2Error::InvalidAuthCode)
                        }
                    } else {
                        Err(OAuth2Error::InvalidCallback(
                            "Missing state parameter".to_string(),
                        ))
                    }
                };

                let _ = tx.send(result);
            }
        });

        Ok((format!("http://127.0.0.1:8080/callback"), rx))
    }

    /// Exchange authorization code for tokens (Google)
    async fn exchange_code_for_token(
        &self,
        auth_code: &str,
        code_verifier: &str,
    ) -> Result<GoogleTokenResponse, OAuth2Error> {
        let mut params = HashMap::new();
        params.insert("code", auth_code);
        params.insert("client_id", &self.client_id);
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("grant_type", "authorization_code");
        params.insert("code_verifier", code_verifier);

        if let Some(ref secret) = self.client_secret {
            params.insert("client_secret", secret);
        }

        log::debug!("Exchanging authorization code for tokens");

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                OAuth2Error::NetworkError(format!("Token exchange request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OAuth2Error::TokenExchangeFailed(format!(
                "HTTP {}: {}",
                status, error_body
            )));
        }

        response.json::<GoogleTokenResponse>().await.map_err(|e| {
            OAuth2Error::TokenExchangeFailed(format!("Failed to parse token response: {}", e))
        })
    }

    /// Exchange authorization code for tokens (Apple)
    async fn exchange_apple_code_for_token(
        &self,
        auth_code: &str,
    ) -> Result<AppleTokenResponse, OAuth2Error> {
        let mut params = HashMap::new();
        params.insert("code", auth_code);
        params.insert("client_id", &self.client_id);
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("grant_type", "authorization_code");

        // Apple requires client_secret for web apps
        // For production, this should be a JWT signed with your private key
        if let Some(ref secret) = self.client_secret {
            params.insert("client_secret", secret);
        } else {
            return Err(OAuth2Error::TokenExchangeFailed(
                "Apple OAuth requires client_secret (JWT)".to_string(),
            ));
        }

        log::debug!("Exchanging Apple authorization code for tokens");

        let response = self
            .http_client
            .post(APPLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                OAuth2Error::NetworkError(format!("Apple token exchange request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OAuth2Error::TokenExchangeFailed(format!(
                "Apple returned HTTP {}: {}",
                status, error_body
            )));
        }

        response.json::<AppleTokenResponse>().await.map_err(|e| {
            OAuth2Error::TokenExchangeFailed(format!(
                "Failed to parse Apple token response: {}",
                e
            ))
        })
    }

    /// Decode Apple ID token (JWT) without verification
    /// NOTE: In production, you MUST verify the JWT signature using Apple's public keys
    /// This is a simplified implementation for educational purposes
    fn decode_apple_id_token(&self, id_token: &str) -> Result<AppleIdTokenClaims, OAuth2Error> {
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err(OAuth2Error::TokenExchangeFailed(
                "Invalid JWT format".to_string(),
            ));
        }

        // Decode the payload (second part)
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| OAuth2Error::TokenExchangeFailed(format!("Failed to decode JWT: {}", e)))?;

        let claims: AppleIdTokenClaims = serde_json::from_slice(&payload_bytes).map_err(|e| {
            OAuth2Error::TokenExchangeFailed(format!("Failed to parse JWT claims: {}", e))
        })?;

        // Basic validation
        if claims.iss != "https://appleid.apple.com" {
            return Err(OAuth2Error::TokenExchangeFailed(
                "Invalid JWT issuer".to_string(),
            ));
        }

        if claims.aud != self.client_id {
            return Err(OAuth2Error::TokenExchangeFailed(
                "JWT audience mismatch".to_string(),
            ));
        }

        // Check expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        if claims.exp < now {
            return Err(OAuth2Error::TokenExchangeFailed(
                "JWT has expired".to_string(),
            ));
        }

        log::debug!(
            "Decoded Apple ID token for user: {} (email: {:?}, private_email: {:?})",
            claims.sub,
            claims.email,
            claims.is_private_email
        );

        Ok(claims)
    }

    /// Refresh access token using refresh token (Google)
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenData, OAuth2Error> {
        let mut params = HashMap::new();
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.client_id);
        params.insert("grant_type", "refresh_token");

        if let Some(ref secret) = self.client_secret {
            params.insert("client_secret", secret);
        }

        log::debug!("Refreshing access token");

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                OAuth2Error::NetworkError(format!("Token refresh request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OAuth2Error::TokenRefreshFailed(format!(
                "HTTP {}: {}",
                status, error_body
            )));
        }

        let token_response: GoogleTokenResponse = response.json().await.map_err(|e| {
            OAuth2Error::TokenRefreshFailed(format!("Failed to parse refresh response: {}", e))
        })?;

        Ok(TokenData::new(
            token_response.access_token,
            token_response
                .refresh_token
                .or(Some(refresh_token.to_string())),
            token_response.token_type,
            token_response.expires_in,
            token_response.scope,
        ))
    }

    /// Refresh Apple access token using refresh token
    /// NOTE: Apple refresh tokens are single-use and a new refresh token is issued with each refresh
    pub async fn refresh_apple_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenData, OAuth2Error> {
        let mut params = HashMap::new();
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.client_id);
        params.insert("grant_type", "refresh_token");

        // Apple requires client_secret for token refresh
        if let Some(ref secret) = self.client_secret {
            params.insert("client_secret", secret);
        } else {
            return Err(OAuth2Error::TokenRefreshFailed(
                "Apple token refresh requires client_secret".to_string(),
            ));
        }

        log::debug!("Refreshing Apple access token");

        let response = self
            .http_client
            .post(APPLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                OAuth2Error::NetworkError(format!("Apple token refresh request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OAuth2Error::TokenRefreshFailed(format!(
                "Apple refresh returned HTTP {}: {}",
                status, error_body
            )));
        }

        let token_response: AppleTokenResponse = response.json().await.map_err(|e| {
            OAuth2Error::TokenRefreshFailed(format!(
                "Failed to parse Apple refresh response: {}",
                e
            ))
        })?;

        // Decode new ID token
        let id_token_claims = self.decode_apple_id_token(&token_response.id_token)?;

        log::debug!(
            "Apple token refreshed successfully for user: {}",
            id_token_claims.sub
        );

        // IMPORTANT: Apple issues a NEW refresh token with each refresh
        // The old refresh token is now invalid
        Ok(TokenData::new(
            token_response.access_token,
            token_response.refresh_token, // This is a NEW refresh token
            token_response.token_type,
            token_response.expires_in,
            Some(format!("apple_user:{}", id_token_claims.sub)),
        ))
    }

    /// Exchange Google tokens with HiNotes backend
    async fn exchange_with_hinotes(
        &self,
        google_access_token: &str,
    ) -> Result<String, OAuth2Error> {
        log::debug!("Exchanging Google token with HiNotes backend");

        let response = self
            .http_client
            .post(&format!("{}/oauth2/signin/google", HINOTES_API_BASE))
            .json(&serde_json::json!({
                "access_token": google_access_token
            }))
            .send()
            .await
            .map_err(|e| {
                OAuth2Error::NetworkError(format!("HiNotes token exchange failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OAuth2Error::ServerError(format!(
                "HiNotes returned HTTP {}: {}",
                status, error_body
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            OAuth2Error::ServerError(format!("Failed to parse HiNotes response: {}", e))
        })?;

        json.get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| OAuth2Error::ServerError("No token in HiNotes response".to_string()))
    }

    /// Exchange Apple tokens with HiNotes backend
    async fn exchange_apple_with_hinotes(
        &self,
        apple_id_token: &str,
        apple_auth_code: &str,
        user_info: Option<&AppleUserInfo>,
    ) -> Result<String, OAuth2Error> {
        log::debug!("Exchanging Apple token with HiNotes backend");

        // Build request payload
        // Apple provides different data on first vs subsequent authentications
        let mut payload = serde_json::json!({
            "id_token": apple_id_token,
            "authorization_code": apple_auth_code,
        });

        // Include user info if provided (first-time auth only)
        if let Some(info) = user_info {
            if let Some(ref email) = info.email {
                payload["email"] = serde_json::json!(email);
            }
            if let Some(ref name) = info.name {
                let mut name_obj = serde_json::json!({});
                if let Some(ref first) = name.first_name {
                    name_obj["firstName"] = serde_json::json!(first);
                }
                if let Some(ref last) = name.last_name {
                    name_obj["lastName"] = serde_json::json!(last);
                }
                payload["name"] = name_obj;
            }
        }

        let response = self
            .http_client
            .post(&format!("{}/oauth2/signin/apple", HINOTES_API_BASE))
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                OAuth2Error::NetworkError(format!(
                    "HiNotes Apple token exchange failed: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OAuth2Error::ServerError(format!(
                "HiNotes Apple exchange returned HTTP {}: {}",
                status, error_body
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            OAuth2Error::ServerError(format!("Failed to parse HiNotes Apple response: {}", e))
        })?;

        json.get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OAuth2Error::ServerError("No token in HiNotes Apple response".to_string())
            })
    }

    /// Authenticate with Google OAuth2
    pub async fn authenticate_google(&self) -> Result<TokenData, OAuth2Error> {
        log::info!("Starting Google OAuth2 authentication");

        // Generate PKCE parameters
        let (code_verifier, code_challenge) = Self::generate_pkce();

        // Generate random state for CSRF protection
        let state: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Start local callback server
        let (_redirect_uri, rx) = Self::start_callback_server(state.clone()).await?;

        // Build authorization URL
        let auth_url = self.build_google_auth_url(&code_challenge, &state);

        // Open browser for user authentication
        log::info!("Opening browser for Google authentication");
        open::that(&auth_url)
            .map_err(|e| OAuth2Error::ServerError(format!("Failed to open browser: {}", e)))?;

        // Wait for callback with timeout
        let auth_code = tokio::time::timeout(Duration::from_secs(AUTH_TIMEOUT_SECS), rx)
            .await
            .map_err(|_| OAuth2Error::Timeout(AUTH_TIMEOUT_SECS))?
            .map_err(|_| OAuth2Error::UserCancelled)??;

        // Exchange authorization code for tokens
        let google_tokens = self
            .exchange_code_for_token(&auth_code, &code_verifier)
            .await?;

        // Exchange Google tokens with HiNotes backend
        let hinotes_token = self
            .exchange_with_hinotes(&google_tokens.access_token)
            .await?;

        log::info!("Google OAuth2 authentication successful");

        // Return both Google tokens (for refresh) and HiNotes token
        Ok(TokenData::new(
            hinotes_token,
            google_tokens.refresh_token,
            google_tokens.token_type,
            google_tokens.expires_in,
            google_tokens.scope,
        ))
    }

    /// Authenticate with Apple OAuth2
    pub async fn authenticate_apple(&self) -> Result<TokenData, OAuth2Error> {
        log::info!("Starting Apple OAuth2 authentication");

        // Generate random state for CSRF protection
        let state: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Start local callback server
        // Note: Apple uses form_post response_mode, so the callback will receive POST data
        let (_redirect_uri, mut rx) = Self::start_callback_server(state.clone()).await?;

        // Build authorization URL
        let auth_url = self.build_apple_auth_url(&state);

        // Open browser for user authentication
        log::info!("Opening browser for Apple authentication");
        open::that(&auth_url)
            .map_err(|e| OAuth2Error::ServerError(format!("Failed to open browser: {}", e)))?;

        // Wait for callback with timeout
        let auth_code = tokio::time::timeout(Duration::from_secs(AUTH_TIMEOUT_SECS), rx)
            .await
            .map_err(|_| OAuth2Error::Timeout(AUTH_TIMEOUT_SECS))?
            .map_err(|_| OAuth2Error::UserCancelled)??;

        log::debug!("Received Apple authorization code");

        // Exchange authorization code for tokens
        let apple_tokens = self.exchange_apple_code_for_token(&auth_code).await?;

        // Decode ID token to extract user information
        let id_token_claims = self.decode_apple_id_token(&apple_tokens.id_token)?;

        // Log user information
        log::info!(
            "Apple authentication successful for user: {} (email: {:?})",
            id_token_claims.sub,
            id_token_claims.email
        );

        // Handle Apple's email relay feature
        if id_token_claims.is_private_email == Some(true) {
            log::info!("User is using Apple's private email relay");
        }

        // Handle real user indicator (anti-fraud)
        match id_token_claims.real_user_status {
            Some(2) => log::debug!("User is likely a real person (high confidence)"),
            Some(1) => log::debug!("User real status unknown"),
            Some(0) => log::warn!("Real user status check unsupported"),
            _ => {}
        }

        // Note: Apple only provides user name and email on the FIRST authorization
        // On subsequent authorizations, you must retrieve this from your own database
        // using the 'sub' (subject) claim as the unique identifier

        // Exchange Apple tokens with HiNotes backend
        // For first-time auth, you would pass user_info here if available from the callback
        let hinotes_token = self
            .exchange_apple_with_hinotes(&apple_tokens.id_token, &auth_code, None)
            .await?;

        log::info!("Apple OAuth2 authentication completed successfully");

        // Return token data
        // Note: Apple refresh tokens are single-use and must be exchanged immediately
        Ok(TokenData::new(
            hinotes_token,
            apple_tokens.refresh_token,
            apple_tokens.token_type,
            apple_tokens.expires_in,
            Some(format!("apple_user:{}", id_token_claims.sub)),
        ))
    }

    /// Get valid access token, refreshing if necessary
    pub async fn get_valid_token(&self, username: &str) -> Result<String, OAuth2Error> {
        let storage = TokenStorage::new(username)
            .map_err(|e| OAuth2Error::ServerError(format!("Storage error: {}", e)))?;

        let tokens = storage
            .retrieve_tokens()
            .map_err(|e| OAuth2Error::ServerError(format!("Failed to retrieve tokens: {}", e)))?;

        if tokens.is_expired() {
            if let Some(ref refresh_token) = tokens.refresh_token {
                log::info!("Access token expired, refreshing");
                let new_tokens = self.refresh_token(refresh_token).await?;
                storage.store_tokens(&new_tokens).map_err(|e| {
                    OAuth2Error::ServerError(format!("Failed to store refreshed tokens: {}", e))
                })?;
                Ok(new_tokens.access_token)
            } else {
                Err(OAuth2Error::TokenRefreshFailed(
                    "No refresh token available".to_string(),
                ))
            }
        } else {
            Ok(tokens.access_token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let (verifier, challenge) = OAuth2Handler::generate_pkce();

        // Verifier should be 128 characters
        assert_eq!(verifier.len(), 128);

        // Challenge should be base64-url encoded (no padding)
        assert!(!challenge.contains('='));
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }

    #[test]
    fn test_google_auth_url_construction() {
        let oauth = OAuth2Handler::new("test-client-id", None);
        let (_, challenge) = OAuth2Handler::generate_pkce();
        let state = "test-state";

        let url = oauth.build_google_auth_url(&challenge, state);

        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("access_type=offline"));
    }

    #[test]
    fn test_oauth2_error_display() {
        let err = OAuth2Error::Timeout(300);
        assert_eq!(err.to_string(), "Authentication timeout after 300 seconds");

        let err = OAuth2Error::UserCancelled;
        assert_eq!(err.to_string(), "User cancelled authentication");

        let err = OAuth2Error::TokenExchangeFailed("network error".to_string());
        assert_eq!(err.to_string(), "Token exchange failed: network error");
    }

    #[tokio::test]
    async fn test_refresh_token_builds_correct_request() {
        // This is a unit test - we don't actually make the request
        let oauth = OAuth2Handler::new("test-client-id", Some("test-secret".to_string()));

        // We can't easily test the actual refresh without mocking the HTTP client,
        // but we can verify the handler is constructed properly
        assert_eq!(oauth.client_id, "test-client-id");
        assert_eq!(oauth.client_secret, Some("test-secret".to_string()));
        assert_eq!(oauth.redirect_uri, "http://localhost:8080/callback");
    }

    #[tokio::test]
    async fn test_apple_oauth_requires_client_secret() {
        let oauth = OAuth2Handler::new("test-client-id", None);

        // Test that Apple token exchange requires client_secret
        let result = oauth.exchange_apple_code_for_token("test_code").await;
        assert!(result.is_err());
        match result {
            Err(OAuth2Error::TokenExchangeFailed(msg)) => {
                assert!(msg.contains("client_secret"));
            }
            _ => panic!("Expected TokenExchangeFailed"),
        }
    }

    #[test]
    fn test_apple_auth_url_construction() {
        let oauth = OAuth2Handler::new("test.client.id", Some("secret".to_string()));
        let state = "test-state";

        let url = oauth.build_apple_auth_url(state);

        assert!(url.contains("client_id=test.client.id"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("response_mode=form_post"));
        assert!(url.contains("scope=name+email"));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("appleid.apple.com"));
    }

    #[test]
    fn test_decode_apple_id_token_invalid_format() {
        let oauth = OAuth2Handler::new("test-client-id", None);
        let result = oauth.decode_apple_id_token("invalid.token");

        assert!(result.is_err());
        match result {
            Err(OAuth2Error::TokenExchangeFailed(msg)) => {
                assert!(msg.contains("Invalid JWT format"));
            }
            _ => panic!("Expected TokenExchangeFailed"),
        }
    }
}
