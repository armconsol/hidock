// Configuration management Tauri commands
// Allows users to set OAuth credentials and other app configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Google OAuth Client ID
    pub google_client_id: Option<String>,
    /// Google OAuth Client Secret (optional)
    pub google_client_secret: Option<String>,
    /// Apple OAuth Client ID (optional)
    pub apple_client_id: Option<String>,
    /// Apple Team ID (optional)
    pub apple_team_id: Option<String>,
    /// Apple Key ID (optional)
    pub apple_key_id: Option<String>,
    /// HiNotes API Base URL
    pub api_base_url: Option<String>,
}

/// Get the path to the config file
fn get_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Failed to get config directory".to_string())?;

    let app_config_dir = config_dir.join("hinotes");

    // Create directory if it doesn't exist
    fs::create_dir_all(&app_config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(app_config_dir.join("config.json"))
}

/// Load configuration from file
#[tauri::command]
pub fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(AppConfig::default());
    }

    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    Ok(config)
}

/// Save configuration to file
#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Get the configuration file path for display in UI
#[tauri::command]
pub fn get_config_file_path() -> Result<String, String> {
    let path = get_config_path()?;
    Ok(path.to_string_lossy().to_string())
}

/// Check if OAuth is configured
#[tauri::command]
pub fn is_oauth_configured() -> Result<bool, String> {
    let config = load_config()?;
    Ok(config.google_client_id.is_some() || config.apple_client_id.is_some())
}

/// OAuth setup instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSetupInstructions {
    pub provider: String,
    pub title: String,
    pub steps: Vec<String>,
    pub required_fields: Vec<String>,
    pub documentation_url: String,
}

/// Get setup instructions for Google OAuth
#[tauri::command]
pub fn get_google_oauth_instructions() -> OAuthSetupInstructions {
    OAuthSetupInstructions {
        provider: "Google".to_string(),
        title: "Google OAuth2 Setup".to_string(),
        steps: vec![
            "1. Go to the Google Cloud Console (https://console.cloud.google.com/)".to_string(),
            "2. Create a new project or select an existing one".to_string(),
            "3. Navigate to 'APIs & Services' > 'Credentials'".to_string(),
            "4. Click 'Create Credentials' > 'OAuth client ID'".to_string(),
            "5. If prompted, configure the OAuth consent screen first:".to_string(),
            "   - Choose 'External' user type".to_string(),
            "   - Fill in app name, support email, and developer contact".to_string(),
            "   - Add scopes: openid, email, profile".to_string(),
            "   - Save and continue".to_string(),
            "6. Back in Credentials, create OAuth client ID:".to_string(),
            "   - Application type: 'Desktop application'".to_string(),
            "   - Name: HiNotes Desktop".to_string(),
            "7. Copy the Client ID (looks like: xxxxx.apps.googleusercontent.com)".to_string(),
            "8. (Optional) Copy the Client Secret".to_string(),
            "9. Add authorized redirect URI: http://localhost:8080/callback".to_string(),
            "10. Click 'Create' and save your credentials".to_string(),
        ],
        required_fields: vec![
            "GOOGLE_CLIENT_ID (required)".to_string(),
            "GOOGLE_CLIENT_SECRET (optional, but recommended)".to_string(),
        ],
        documentation_url: "https://developers.google.com/identity/protocols/oauth2/native-app".to_string(),
    }
}

/// Get setup instructions for Apple Sign In
#[tauri::command]
pub fn get_apple_oauth_instructions() -> OAuthSetupInstructions {
    OAuthSetupInstructions {
        provider: "Apple".to_string(),
        title: "Apple Sign In Setup".to_string(),
        steps: vec![
            "1. Go to Apple Developer Portal (https://developer.apple.com/)".to_string(),
            "2. Sign in with your Apple Developer account".to_string(),
            "3. Navigate to 'Certificates, Identifiers & Profiles'".to_string(),
            "4. Create an App ID:".to_string(),
            "   - Click 'Identifiers' > '+' button".to_string(),
            "   - Select 'App IDs' > 'App'".to_string(),
            "   - Description: HiNotes Desktop".to_string(),
            "   - Bundle ID: com.yourcompany.hinotes (explicit)".to_string(),
            "   - Enable 'Sign In with Apple' capability".to_string(),
            "5. Create a Service ID (for web authentication):".to_string(),
            "   - Click 'Identifiers' > '+' button".to_string(),
            "   - Select 'Services IDs'".to_string(),
            "   - Identifier: com.yourcompany.hinotes.signin".to_string(),
            "   - Enable 'Sign In with Apple'".to_string(),
            "   - Configure: Add domain and return URL (http://localhost:8080/callback)".to_string(),
            "6. Create a Key for Sign In with Apple:".to_string(),
            "   - Click 'Keys' > '+' button".to_string(),
            "   - Key Name: HiNotes Sign In Key".to_string(),
            "   - Enable 'Sign In with Apple'".to_string(),
            "   - Configure: Select your App ID".to_string(),
            "   - Download the .p8 key file (you can only download it once!)".to_string(),
            "   - Note the Key ID (10-character string)".to_string(),
            "7. Find your Team ID:".to_string(),
            "   - In Apple Developer Portal, top-right corner".to_string(),
            "   - Or in Membership section (10-character string)".to_string(),
            "8. Save all credentials:".to_string(),
            "   - Service ID (APPLE_CLIENT_ID)".to_string(),
            "   - Team ID (APPLE_TEAM_ID)".to_string(),
            "   - Key ID (APPLE_KEY_ID)".to_string(),
            "   - Keep the .p8 key file secure".to_string(),
        ],
        required_fields: vec![
            "APPLE_CLIENT_ID (Service ID)".to_string(),
            "APPLE_TEAM_ID (10-character Team ID)".to_string(),
            "APPLE_KEY_ID (10-character Key ID)".to_string(),
            "Apple .p8 key file (not stored in config)".to_string(),
        ],
        documentation_url: "https://developer.apple.com/documentation/sign_in_with_apple".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            google_client_id: Some("test-client-id".to_string()),
            google_client_secret: Some("test-secret".to_string()),
            apple_client_id: None,
            apple_team_id: None,
            apple_key_id: None,
            api_base_url: Some("https://hinotes.hidock.com/v1".to_string()),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.google_client_id, deserialized.google_client_id);
        assert_eq!(config.google_client_secret, deserialized.google_client_secret);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.google_client_id.is_none());
        assert!(config.apple_client_id.is_none());
        assert!(config.api_base_url.is_none());
    }
}
