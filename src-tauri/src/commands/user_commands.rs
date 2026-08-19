// User Profile Management Tauri commands

use crate::api::types::UserInfo;
use crate::commands::auth_commands::AuthState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub region: Option<String>,
}

impl From<UserInfo> for UserProfile {
    fn from(user: UserInfo) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: None,
            region: None,
        }
    }
}

/// Get current user profile
///
/// Fetches user information from the API and combines with local settings
///
/// # Arguments
/// * `state` - Application state containing API client
///
/// # Returns
/// Result containing user profile data
#[tauri::command]
pub async fn get_user_profile(state: State<'_, AuthState>) -> Result<UserProfile, String> {
    let client = state.api_client.read().await;

    let user_info = client
        .get_user_info()
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?;

    Ok(UserProfile::from(user_info))
}

/// Update user display name
///
/// # Arguments
/// * `new_name` - New display name for the user
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with updated user profile on success
#[tauri::command]
pub async fn update_user_name(
    new_name: String,
    state: State<'_, AuthState>,
) -> Result<UserProfile, String> {
    if new_name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    let client = state.api_client.read().await;

    client
        .rename_user(new_name.clone())
        .await
        .map_err(|e| format!("Failed to update user name: {}", e))?;

    // Fetch updated user info
    let user_info = client
        .get_user_info()
        .await
        .map_err(|e| format!("Failed to fetch updated user info: {}", e))?;

    Ok(UserProfile::from(user_info))
}

/// Update user region/locale
///
/// # Arguments
/// * `region` - Region/locale code (e.g., "en-US", "zh-CN", "ja-JP")
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
#[tauri::command]
pub async fn update_user_region(
    region: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if region.trim().is_empty() {
        return Err("Region cannot be empty".to_string());
    }

    let client = state.api_client.read().await;

    client
        .update_user_region(region.clone())
        .await
        .map_err(|e| format!("Failed to update user region: {}", e))?;

    Ok(format!("Region updated to: {}", region))
}

/// Upload user profile avatar
///
/// # Arguments
/// * `image_path` - Absolute path to the image file
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with avatar URL on success
#[tauri::command]
pub async fn upload_user_avatar(
    image_path: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    // Read the image file
    let image_data =
        std::fs::read(&image_path).map_err(|e| format!("Failed to read image file: {}", e))?;

    // Extract filename from path
    let filename = std::path::Path::new(&image_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file path".to_string())?
        .to_string();

    let client = state.api_client.read().await;

    let avatar_url = client
        .upload_avatar(image_data, filename)
        .await
        .map_err(|e| format!("Failed to upload avatar: {}", e))?;

    Ok(avatar_url)
}

// ===== PASSWORD AND SECURITY COMMANDS =====

/// Change user password
///
/// # Arguments
/// * `current_password` - Current password for verification
/// * `new_password` - New password to set
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
///
/// # Side Effects
/// Forces logout after successful password change for security
#[tauri::command]
pub async fn change_password(
    current_password: String,
    new_password: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if current_password.is_empty() {
        return Err("Current password cannot be empty".to_string());
    }

    if new_password.len() < 8 {
        return Err("New password must be at least 8 characters long".to_string());
    }

    if current_password == new_password {
        return Err("New password must be different from current password".to_string());
    }

    let client = state.api_client.read().await;

    client
        .update_password(&current_password, &new_password)
        .await
        .map_err(|e| format!("Failed to change password: {}", e))?;

    // Force logout for security after password change
    client
        .logout()
        .await
        .map_err(|e| format!("Password changed but logout failed: {}", e))?;

    Ok("Password changed successfully. Please log in again.".to_string())
}

/// Delete user account
///
/// # Arguments
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
///
/// # Side Effects
/// Clears all local data after successful account deletion
///
/// # Warning
/// This action is irreversible. All user data will be permanently deleted.
#[tauri::command]
pub async fn delete_user_account(state: State<'_, AuthState>) -> Result<String, String> {
    let client = state.api_client.read().await;

    client
        .delete_user_account()
        .await
        .map_err(|e| format!("Failed to delete account: {}", e))?;

    // Clear local data after account deletion
    client
        .logout()
        .await
        .map_err(|e| format!("Account deleted but local cleanup failed: {}", e))?;

    Ok("Account deleted successfully".to_string())
}

/// Send email verification code
///
/// # Arguments
/// * `email` - Email address to send verification code to
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
#[tauri::command]
pub async fn send_email_verification(
    email: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }

    let client = state.api_client.read().await;

    client
        .send_email_verification(&email)
        .await
        .map_err(|e| format!("Failed to send verification code: {}", e))?;

    Ok(format!("Verification code sent to {}", email))
}

/// Verify email verification code
///
/// # Arguments
/// * `code` - Verification code received via email
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
#[tauri::command]
pub async fn verify_email_code(
    code: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if code.is_empty() {
        return Err("Verification code cannot be empty".to_string());
    }

    let client = state.api_client.read().await;

    client
        .verify_email_code(&code)
        .await
        .map_err(|e| format!("Failed to verify email: {}", e))?;

    Ok("Email verified successfully".to_string())
}

/// Send password reset code
///
/// # Arguments
/// * `email` - Email address to send reset code to
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
#[tauri::command]
pub async fn send_password_reset(
    email: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }

    let client = state.api_client.read().await;

    client
        .send_password_reset_code(&email)
        .await
        .map_err(|e| format!("Failed to send reset code: {}", e))?;

    Ok(format!("Password reset code sent to {}", email))
}

/// Verify password reset code
///
/// # Arguments
/// * `code` - Reset code received via email
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
#[tauri::command]
pub async fn verify_reset_code(
    code: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if code.is_empty() {
        return Err("Reset code cannot be empty".to_string());
    }

    let client = state.api_client.read().await;

    client
        .verify_password_reset_code(&code)
        .await
        .map_err(|e| format!("Failed to verify reset code: {}", e))?;

    Ok("Reset code verified successfully".to_string())
}

/// Save new password after reset
///
/// # Arguments
/// * `code` - Verified reset code
/// * `password` - New password to set
/// * `state` - Application state containing API client
///
/// # Returns
/// Result with success message
#[tauri::command]
pub async fn save_new_password(
    code: String,
    password: String,
    state: State<'_, AuthState>,
) -> Result<String, String> {
    if code.is_empty() {
        return Err("Reset code cannot be empty".to_string());
    }

    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    let client = state.api_client.read().await;

    client
        .save_new_password(&code, &password)
        .await
        .map_err(|e| format!("Failed to save new password: {}", e))?;

    Ok("Password reset successfully. Please log in with your new password.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_from_user_info() {
        let user_info = UserInfo {
            id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
        };

        let profile = UserProfile::from(user_info);

        assert_eq!(profile.id, "user-123");
        assert_eq!(profile.email, "test@example.com");
        assert_eq!(profile.name, "Test User");
        assert!(profile.avatar_url.is_none());
        assert!(profile.region.is_none());
    }

    #[test]
    fn test_user_profile_serialization() {
        let profile = UserProfile {
            id: "user-456".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            region: Some("en-US".to_string()),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: UserProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "user-456");
        assert_eq!(deserialized.email, "user@example.com");
        assert_eq!(deserialized.name, "John Doe");
        assert_eq!(
            deserialized.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert_eq!(deserialized.region, Some("en-US".to_string()));
    }
}
