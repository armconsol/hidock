use crate::commands::AppState;
use crate::referral::rewards::{Reward, RewardHistory, RewardStatus, RewardsManager};
use tauri::State;

/// List all rewards with optional status filter
#[tauri::command]
pub async fn list_rewards(
    status_filter: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Reward>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    let manager = RewardsManager::new(db_path.to_str().unwrap());

    let status = status_filter.and_then(|s| match s.to_lowercase().as_str() {
        "available" => Some(RewardStatus::Available),
        "redeemed" => Some(RewardStatus::Redeemed),
        "expired" => Some(RewardStatus::Expired),
        "pending" => Some(RewardStatus::Pending),
        _ => None,
    });

    manager
        .list_rewards(status)
        .map_err(|e| format!("Failed to list rewards: {}", e))
}

/// Redeem a reward by ID
#[tauri::command]
pub async fn redeem_reward(
    reward_id: String,
    points_to_use: f64,
    state: State<'_, AppState>,
) -> Result<RewardHistory, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    let manager = RewardsManager::new(db_path.to_str().unwrap());

    manager
        .redeem_reward(&reward_id, points_to_use)
        .map_err(|e| format!("Failed to redeem reward: {}", e))
}

/// Request a PayPal payout
#[tauri::command]
pub async fn request_payout(
    amount: f64,
    paypal_email: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    let manager = RewardsManager::new(db_path.to_str().unwrap());

    // Minimum payout threshold: $10
    let minimum_threshold = 10.0;

    manager
        .request_payout(amount, &paypal_email, minimum_threshold)
        .map_err(|e| format!("Failed to request payout: {}", e))
}

/// Get reward history
#[tauri::command]
pub async fn get_reward_history(
    reward_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<RewardHistory>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    let manager = RewardsManager::new(db_path.to_str().unwrap());

    manager
        .get_reward_history(reward_id.as_deref(), limit)
        .map_err(|e| format!("Failed to get reward history: {}", e))
}

/// Mark expired rewards
#[tauri::command]
pub async fn expire_rewards(state: State<'_, AppState>) -> Result<usize, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    let manager = RewardsManager::new(db_path.to_str().unwrap());

    manager
        .expire_rewards()
        .map_err(|e| format!("Failed to expire rewards: {}", e))
}

/// Add a reward (for admin/testing purposes)
#[tauri::command]
pub async fn add_reward(reward: Reward, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    let manager = RewardsManager::new(db_path.to_str().unwrap());

    manager
        .add_reward(&reward)
        .map_err(|e| format!("Failed to add reward: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::referral::rewards::{RewardType, RewardsManager};
    use chrono::Utc;
    use std::sync::Mutex;

    fn create_test_state() -> AppState {
        let db = Database::new_in_memory().expect("Failed to create test database");
        AppState { db: Mutex::new(db) }
    }

    fn create_test_reward(id: &str) -> Reward {
        Reward {
            id: id.to_string(),
            reward_type: RewardType::Minutes,
            amount: 60.0,
            description: "Test reward".to_string(),
            expires_at: None,
            status: RewardStatus::Available,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_list_rewards_empty() {
        let state = create_test_state();
        let state_wrapper = State::from(&state);

        let result = list_rewards(None, state_wrapper).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_list_rewards() {
        let state = create_test_state();

        // Initialize rewards database
        {
            let db = state.db.lock().unwrap();
            let db_path = db.get_db_path();
            let manager = RewardsManager::new(db_path.to_str().unwrap());
            manager.init_database().unwrap();
        }

        // Add a reward
        let reward = create_test_reward("reward-1");
        let state_wrapper = State::from(&state);
        add_reward(reward.clone(), state_wrapper).await.unwrap();

        // List rewards
        let state_wrapper2 = State::from(&state);
        let result = list_rewards(None, state_wrapper2).await;
        assert!(result.is_ok());

        let rewards = result.unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].id, "reward-1");
    }

    #[tokio::test]
    async fn test_redeem_reward() {
        let state = create_test_state();

        // Initialize and add reward
        {
            let db = state.db.lock().unwrap();
            let db_path = db.get_db_path();
            let manager = RewardsManager::new(db_path.to_str().unwrap());
            manager.init_database().unwrap();

            let reward = create_test_reward("reward-1");
            manager.add_reward(&reward).unwrap();
        }

        // Redeem the reward
        let state_wrapper = State::from(&state);
        let result = redeem_reward("reward-1".to_string(), 100.0, state_wrapper).await;

        assert!(result.is_ok());
        let history = result.unwrap();
        assert_eq!(history.reward_id, "reward-1");
        assert_eq!(history.points_used, 100.0);
    }

    #[tokio::test]
    async fn test_get_reward_history_empty() {
        let state = create_test_state();

        // Initialize database
        {
            let db = state.db.lock().unwrap();
            let db_path = db.get_db_path();
            let manager = RewardsManager::new(db_path.to_str().unwrap());
            manager.init_database().unwrap();
        }

        let state_wrapper = State::from(&state);
        let result = get_reward_history(None, None, state_wrapper).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
