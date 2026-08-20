use crate::commands::AppState;
use crate::referral::{ReferralCode, ReferralStats, ReferralUsage, RewardConfig};
use tauri::State;

/// Generate a new referral code for the current user
#[tauri::command]
pub async fn create_referral_code(
    user_id: String,
    expires_at: Option<String>,
    state: State<'_, AppState>,
) -> Result<ReferralCode, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // Parse expiry date if provided
    let expires_at_parsed = if let Some(exp_str) = expires_at {
        let dt = chrono::DateTime::parse_from_rfc3339(&exp_str)
            .map_err(|e| format!("Invalid expiry date: {}", e))?;
        Some(dt.with_timezone(&chrono::Utc))
    } else {
        None
    };

    db.generate_referral_code_with_expiry(&user_id, expires_at_parsed)
        .map_err(|e| format!("Failed to create referral code: {}", e))
}

/// Get referral statistics for a user
#[tauri::command]
pub async fn get_referral_stats(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<ReferralStats, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_referral_stats(&user_id)
        .map_err(|e| format!("Failed to get referral stats: {}", e))
}

/// Track referral code usage when a new user signs up
#[tauri::command]
pub async fn track_referral_usage(
    code: String,
    new_user_id: String,
    state: State<'_, AppState>,
) -> Result<ReferralUsage, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let reward_config = RewardConfig::default();

    db.apply_referral_code(&new_user_id, &code, &reward_config)
        .map_err(|e| format!("Failed to track referral: {}", e))
}

/// List all referral codes for a user
#[tauri::command]
pub async fn get_user_referral_codes(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReferralCode>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_user_referral_codes(&user_id)
        .map_err(|e| format!("Failed to list referral codes: {}", e))
}

/// Validate a referral code
#[tauri::command]
pub async fn validate_referral_code(
    code: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.validate_referral_code(&code)
        .map_err(|e| format!("Failed to validate referral code: {}", e))
}

/// Deactivate a referral code
#[tauri::command]
pub async fn deactivate_referral_code(
    code_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_referral_code(&code_id)
        .map_err(|e| format!("Failed to deactivate referral code: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::sync::Mutex;

    fn create_test_state() -> AppState {
        let db = Database::new_in_memory().expect("Failed to create test database");
        AppState { db: Mutex::new(db) }
    }

    #[tokio::test]
    async fn test_create_referral_code() {
        let state = create_test_state();
        let db = state.db.lock().unwrap();

        let result = db.generate_referral_code_with_expiry("user-123", None);

        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code.user_id, "user-123");
        assert_eq!(code.code.len(), 8);
    }

    #[tokio::test]
    async fn test_get_referral_stats_empty() {
        let state = create_test_state();
        let db = state.db.lock().unwrap();

        let result = db.get_referral_stats("user-123");

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_referrals, 0);
        assert_eq!(stats.total_reward_points, 0);
    }

    #[tokio::test]
    async fn test_validate_referral_code() {
        let state = create_test_state();
        let db = state.db.lock().unwrap();

        // Create a code first
        let code_result = db
            .generate_referral_code_with_expiry("user-123", None)
            .unwrap();

        // Validate it
        let is_valid = db.validate_referral_code(&code_result.code).unwrap();

        assert!(is_valid);
    }
}
