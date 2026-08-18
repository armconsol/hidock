use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reward {
    pub id: String,
    pub reward_type: RewardType,
    pub amount: f64,
    pub description: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: RewardStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RewardType {
    Minutes,   // Transcription minutes
    Cash,      // PayPal cash payout
    Credit,    // Account credit
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RewardStatus {
    Available,
    Redeemed,
    Expired,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardHistory {
    pub id: String,
    pub reward_id: String,
    pub action: RewardAction,
    pub points_used: f64,
    pub occurred_at: DateTime<Utc>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RewardAction {
    Redeemed,
    Expired,
    PayoutRequested,
    PayoutCompleted,
    PayoutFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRequest {
    pub amount: f64,
    pub paypal_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub payout_id: String,
    pub status: String,
    pub amount: f64,
    pub estimated_completion: Option<DateTime<Utc>>,
}

pub struct RewardsManager {
    db_path: String,
}

impl RewardsManager {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }

    fn get_connection(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    /// Initialize rewards database tables
    pub fn init_database(&self) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rewards (
                id TEXT PRIMARY KEY,
                reward_type TEXT CHECK(reward_type IN ('minutes', 'cash', 'credit')) NOT NULL,
                amount REAL NOT NULL,
                description TEXT NOT NULL,
                expires_at DATETIME,
                status TEXT CHECK(status IN ('available', 'redeemed', 'expired', 'pending')) NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS reward_history (
                id TEXT PRIMARY KEY,
                reward_id TEXT NOT NULL,
                action TEXT CHECK(action IN ('redeemed', 'expired', 'payout_requested', 'payout_completed', 'payout_failed')) NOT NULL,
                points_used REAL NOT NULL,
                occurred_at DATETIME NOT NULL,
                details TEXT,
                FOREIGN KEY (reward_id) REFERENCES rewards(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rewards_status ON rewards(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rewards_expires ON rewards(expires_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_reward_history_reward ON reward_history(reward_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_reward_history_occurred ON reward_history(occurred_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// List all available rewards
    pub fn list_rewards(&self, status_filter: Option<RewardStatus>) -> Result<Vec<Reward>> {
        let conn = self.get_connection()?;

        let query = if let Some(status) = status_filter {
            format!(
                "SELECT id, reward_type, amount, description, expires_at, status, created_at
                 FROM rewards WHERE status = '{}' ORDER BY created_at DESC",
                match status {
                    RewardStatus::Available => "available",
                    RewardStatus::Redeemed => "redeemed",
                    RewardStatus::Expired => "expired",
                    RewardStatus::Pending => "pending",
                }
            )
        } else {
            "SELECT id, reward_type, amount, description, expires_at, status, created_at
             FROM rewards ORDER BY created_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let rewards = stmt
            .query_map([], |row| {
                Ok(Reward {
                    id: row.get(0)?,
                    reward_type: match row.get::<_, String>(1)?.as_str() {
                        "minutes" => RewardType::Minutes,
                        "cash" => RewardType::Cash,
                        "credit" => RewardType::Credit,
                        _ => RewardType::Minutes,
                    },
                    amount: row.get(2)?,
                    description: row.get(3)?,
                    expires_at: row.get::<_, Option<String>>(4)?.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                    status: match row.get::<_, String>(5)?.as_str() {
                        "available" => RewardStatus::Available,
                        "redeemed" => RewardStatus::Redeemed,
                        "expired" => RewardStatus::Expired,
                        "pending" => RewardStatus::Pending,
                        _ => RewardStatus::Available,
                    },
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rewards)
    }

    /// Redeem a reward by ID
    pub fn redeem_reward(&self, reward_id: &str, points_to_use: f64) -> Result<RewardHistory> {
        if points_to_use <= 0.0 {
            anyhow::bail!("Points to use must be greater than zero");
        }

        let conn = self.get_connection()?;

        // Check if reward exists and is available
        let reward: Option<(String, String)> = conn
            .query_row(
                "SELECT id, status FROM rewards WHERE id = ?1",
                [reward_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        match reward {
            None => anyhow::bail!("Reward not found"),
            Some((_, status)) if status != "available" => {
                anyhow::bail!("Reward is not available for redemption")
            }
            Some(_) => {}
        }

        // Check if reward has expired
        let expires_at: Option<String> = conn.query_row(
            "SELECT expires_at FROM rewards WHERE id = ?1",
            [reward_id],
            |row| row.get(0),
        )?;

        if let Some(exp_str) = expires_at {
            if let Ok(exp_date) = DateTime::parse_from_rfc3339(&exp_str) {
                if exp_date.with_timezone(&Utc) < Utc::now() {
                    anyhow::bail!("Reward has expired");
                }
            }
        }

        // Update reward status
        conn.execute(
            "UPDATE rewards SET status = 'redeemed', updated_at = ?1 WHERE id = ?2",
            [Utc::now().to_rfc3339(), reward_id.to_string()],
        )?;

        // Create history entry
        let history_id = uuid::Uuid::new_v4().to_string();
        let occurred_at = Utc::now();

        conn.execute(
            "INSERT INTO reward_history (id, reward_id, action, points_used, occurred_at)
             VALUES (?1, ?2, 'redeemed', ?3, ?4)",
            [
                &history_id,
                reward_id,
                &points_to_use.to_string(),
                &occurred_at.to_rfc3339(),
            ],
        )?;

        Ok(RewardHistory {
            id: history_id,
            reward_id: reward_id.to_string(),
            action: RewardAction::Redeemed,
            points_used: points_to_use,
            occurred_at,
            details: None,
        })
    }

    /// Request a PayPal payout
    pub fn request_payout(
        &self,
        amount: f64,
        paypal_email: &str,
        minimum_threshold: f64,
    ) -> Result<String> {
        if amount < minimum_threshold {
            anyhow::bail!(
                "Payout amount ${:.2} is below minimum threshold ${:.2}",
                amount,
                minimum_threshold
            );
        }

        if paypal_email.is_empty() {
            anyhow::bail!("PayPal email is required");
        }

        if !paypal_email.contains('@') {
            anyhow::bail!("Invalid PayPal email format");
        }

        let conn = self.get_connection()?;

        // Create a pending payout reward entry
        let reward_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        conn.execute(
            "INSERT INTO rewards (id, reward_type, amount, description, status, created_at, updated_at)
             VALUES (?1, 'cash', ?2, ?3, 'pending', ?4, ?5)",
            [
                &reward_id,
                &amount.to_string(),
                &format!("PayPal payout to {}", paypal_email),
                &now.to_rfc3339(),
                &now.to_rfc3339(),
            ],
        )?;

        // Create history entry for payout request
        let history_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO reward_history (id, reward_id, action, points_used, occurred_at, details)
             VALUES (?1, ?2, 'payout_requested', ?3, ?4, ?5)",
            [
                &history_id,
                &reward_id,
                &amount.to_string(),
                &now.to_rfc3339(),
                &format!("{{\"paypal_email\": \"{}\"}}", paypal_email),
            ],
        )?;

        Ok(reward_id)
    }

    /// Get reward history
    pub fn get_reward_history(
        &self,
        reward_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<RewardHistory>> {
        let conn = self.get_connection()?;

        let query = if let Some(rid) = reward_id {
            format!(
                "SELECT id, reward_id, action, points_used, occurred_at, details
                 FROM reward_history WHERE reward_id = '{}' ORDER BY occurred_at DESC {}",
                rid,
                limit
                    .map(|l| format!("LIMIT {}", l))
                    .unwrap_or_default()
            )
        } else {
            format!(
                "SELECT id, reward_id, action, points_used, occurred_at, details
                 FROM reward_history ORDER BY occurred_at DESC {}",
                limit
                    .map(|l| format!("LIMIT {}", l))
                    .unwrap_or_default()
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let history = stmt
            .query_map([], |row| {
                Ok(RewardHistory {
                    id: row.get(0)?,
                    reward_id: row.get(1)?,
                    action: match row.get::<_, String>(2)?.as_str() {
                        "redeemed" => RewardAction::Redeemed,
                        "expired" => RewardAction::Expired,
                        "payout_requested" => RewardAction::PayoutRequested,
                        "payout_completed" => RewardAction::PayoutCompleted,
                        "payout_failed" => RewardAction::PayoutFailed,
                        _ => RewardAction::Redeemed,
                    },
                    points_used: row.get(3)?,
                    occurred_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    details: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(history)
    }

    /// Mark expired rewards
    pub fn expire_rewards(&self) -> Result<usize> {
        let conn = self.get_connection()?;
        let now = Utc::now().to_rfc3339();

        // Find rewards that have expired
        let mut stmt = conn.prepare(
            "SELECT id FROM rewards
             WHERE status = 'available'
             AND expires_at IS NOT NULL
             AND expires_at < ?1",
        )?;

        let expired_ids: Vec<String> = stmt
            .query_map([&now], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let count = expired_ids.len();

        // Update status and create history entries
        for reward_id in expired_ids {
            conn.execute(
                "UPDATE rewards SET status = 'expired', updated_at = ?1 WHERE id = ?2",
                [&now, &reward_id],
            )?;

            let history_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO reward_history (id, reward_id, action, points_used, occurred_at)
                 VALUES (?1, ?2, 'expired', 0.0, ?3)",
                [&history_id, &reward_id, &now],
            )?;
        }

        Ok(count)
    }

    /// Add a new reward (for testing or API sync)
    pub fn add_reward(&self, reward: &Reward) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT INTO rewards (id, reward_type, amount, description, expires_at, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &reward.id,
                &match reward.reward_type {
                    RewardType::Minutes => "minutes",
                    RewardType::Cash => "cash",
                    RewardType::Credit => "credit",
                }.to_string(),
                &reward.amount.to_string(),
                &reward.description,
                &reward.expires_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &match reward.status {
                    RewardStatus::Available => "available",
                    RewardStatus::Redeemed => "redeemed",
                    RewardStatus::Expired => "expired",
                    RewardStatus::Pending => "pending",
                }.to_string(),
                &reward.created_at.to_rfc3339(),
                &Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_manager() -> (RewardsManager, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = RewardsManager::new(temp_file.path().to_str().unwrap());
        manager.init_database().unwrap();
        (manager, temp_file)
    }

    fn create_test_reward(id: &str, reward_type: RewardType, amount: f64) -> Reward {
        Reward {
            id: id.to_string(),
            reward_type,
            amount,
            description: format!("Test reward: {} {}", amount, match reward_type {
                RewardType::Minutes => "minutes",
                RewardType::Cash => "USD",
                RewardType::Credit => "credits",
            }),
            expires_at: None,
            status: RewardStatus::Available,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_init_database_creates_tables() {
        let (manager, _temp) = create_test_manager();
        let conn = manager.get_connection().unwrap();

        // Verify rewards table exists
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='rewards'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // Verify reward_history table exists
        let history_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='reward_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(history_exists, 1);
    }

    #[test]
    fn test_list_rewards_empty() {
        let (manager, _temp) = create_test_manager();
        let rewards = manager.list_rewards(None).unwrap();
        assert_eq!(rewards.len(), 0);
    }

    #[test]
    fn test_add_and_list_rewards() {
        let (manager, _temp) = create_test_manager();

        let reward1 = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        let reward2 = create_test_reward("reward-2", RewardType::Cash, 10.0);

        manager.add_reward(&reward1).unwrap();
        manager.add_reward(&reward2).unwrap();

        let rewards = manager.list_rewards(None).unwrap();
        assert_eq!(rewards.len(), 2);
        assert!(rewards.iter().any(|r| r.id == "reward-1"));
        assert!(rewards.iter().any(|r| r.id == "reward-2"));
    }

    #[test]
    fn test_list_rewards_with_status_filter() {
        let (manager, _temp) = create_test_manager();

        let mut reward1 = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        reward1.status = RewardStatus::Available;

        let mut reward2 = create_test_reward("reward-2", RewardType::Cash, 10.0);
        reward2.status = RewardStatus::Redeemed;

        manager.add_reward(&reward1).unwrap();
        manager.add_reward(&reward2).unwrap();

        let available = manager
            .list_rewards(Some(RewardStatus::Available))
            .unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "reward-1");

        let redeemed = manager
            .list_rewards(Some(RewardStatus::Redeemed))
            .unwrap();
        assert_eq!(redeemed.len(), 1);
        assert_eq!(redeemed[0].id, "reward-2");
    }

    #[test]
    fn test_redeem_reward_success() {
        let (manager, _temp) = create_test_manager();

        let reward = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        manager.add_reward(&reward).unwrap();

        let history = manager.redeem_reward("reward-1", 100.0).unwrap();

        assert_eq!(history.reward_id, "reward-1");
        assert_eq!(history.action, RewardAction::Redeemed);
        assert_eq!(history.points_used, 100.0);

        // Verify reward status changed
        let rewards = manager.list_rewards(None).unwrap();
        assert_eq!(rewards[0].status, RewardStatus::Redeemed);
    }

    #[test]
    fn test_redeem_reward_not_found() {
        let (manager, _temp) = create_test_manager();

        let result = manager.redeem_reward("nonexistent", 100.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_redeem_reward_already_redeemed() {
        let (manager, _temp) = create_test_manager();

        let mut reward = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        reward.status = RewardStatus::Redeemed;
        manager.add_reward(&reward).unwrap();

        let result = manager.redeem_reward("reward-1", 100.0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not available for redemption"));
    }

    #[test]
    fn test_redeem_reward_expired() {
        let (manager, _temp) = create_test_manager();

        let mut reward = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        reward.expires_at = Some(Utc::now() - chrono::Duration::days(1));
        manager.add_reward(&reward).unwrap();

        let result = manager.redeem_reward("reward-1", 100.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
    }

    #[test]
    fn test_redeem_reward_invalid_points() {
        let (manager, _temp) = create_test_manager();

        let reward = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        manager.add_reward(&reward).unwrap();

        let result = manager.redeem_reward("reward-1", 0.0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than zero"));
    }

    #[test]
    fn test_request_payout_success() {
        let (manager, _temp) = create_test_manager();

        let reward_id = manager
            .request_payout(25.0, "user@example.com", 10.0)
            .unwrap();

        assert!(!reward_id.is_empty());

        // Verify reward was created
        let rewards = manager
            .list_rewards(Some(RewardStatus::Pending))
            .unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].id, reward_id);
        assert_eq!(rewards[0].reward_type, RewardType::Cash);
        assert_eq!(rewards[0].amount, 25.0);
    }

    #[test]
    fn test_request_payout_below_threshold() {
        let (manager, _temp) = create_test_manager();

        let result = manager.request_payout(5.0, "user@example.com", 10.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("below minimum"));
    }

    #[test]
    fn test_request_payout_empty_email() {
        let (manager, _temp) = create_test_manager();

        let result = manager.request_payout(25.0, "", 10.0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("email is required"));
    }

    #[test]
    fn test_request_payout_invalid_email() {
        let (manager, _temp) = create_test_manager();

        let result = manager.request_payout(25.0, "invalid-email", 10.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_get_reward_history_empty() {
        let (manager, _temp) = create_test_manager();

        let history = manager.get_reward_history(None, None).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_get_reward_history_after_redemption() {
        let (manager, _temp) = create_test_manager();

        let reward = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        manager.add_reward(&reward).unwrap();
        manager.redeem_reward("reward-1", 100.0).unwrap();

        let history = manager.get_reward_history(Some("reward-1"), None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].reward_id, "reward-1");
        assert_eq!(history[0].action, RewardAction::Redeemed);
    }

    #[test]
    fn test_get_reward_history_with_limit() {
        let (manager, _temp) = create_test_manager();

        // Create multiple rewards and redeem them
        for i in 1..=5 {
            let reward = create_test_reward(&format!("reward-{}", i), RewardType::Minutes, 60.0);
            manager.add_reward(&reward).unwrap();
            manager
                .redeem_reward(&format!("reward-{}", i), 100.0)
                .unwrap();
        }

        let history = manager.get_reward_history(None, Some(3)).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_expire_rewards() {
        let (manager, _temp) = create_test_manager();

        // Create expired reward
        let mut reward1 = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        reward1.expires_at = Some(Utc::now() - chrono::Duration::days(1));
        manager.add_reward(&reward1).unwrap();

        // Create non-expired reward
        let mut reward2 = create_test_reward("reward-2", RewardType::Minutes, 60.0);
        reward2.expires_at = Some(Utc::now() + chrono::Duration::days(30));
        manager.add_reward(&reward2).unwrap();

        let count = manager.expire_rewards().unwrap();
        assert_eq!(count, 1);

        // Verify status changed
        let expired = manager.list_rewards(Some(RewardStatus::Expired)).unwrap();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, "reward-1");

        let available = manager
            .list_rewards(Some(RewardStatus::Available))
            .unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "reward-2");
    }

    #[test]
    fn test_expire_rewards_creates_history() {
        let (manager, _temp) = create_test_manager();

        let mut reward = create_test_reward("reward-1", RewardType::Minutes, 60.0);
        reward.expires_at = Some(Utc::now() - chrono::Duration::days(1));
        manager.add_reward(&reward).unwrap();

        manager.expire_rewards().unwrap();

        let history = manager.get_reward_history(Some("reward-1"), None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].action, RewardAction::Expired);
    }
}
