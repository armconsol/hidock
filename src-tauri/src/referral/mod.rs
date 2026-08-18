// Referral system module
pub mod rewards;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferralCode {
    pub id: String,
    pub user_id: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferralUsage {
    pub id: i64,
    pub code_id: String,
    pub referred_user_id: String,
    pub referrer_user_id: String,
    pub applied_at: DateTime<Utc>,
    pub reward_points: i32,
    pub reward_credits: Option<i32>,
    pub reward_subscription_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralStats {
    pub total_referrals: i64,
    pub active_referrals: i64,
    pub total_reward_points: i32,
    pub total_reward_credits: i32,
    pub total_subscription_days: i32,
    pub referral_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardConfig {
    pub points: i32,
    pub credits: Option<i32>,
    pub subscription_days: Option<i32>,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            points: 100,
            credits: Some(50),
            subscription_days: Some(7),
        }
    }
}

pub mod generator {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    const CODE_LENGTH: usize = 8;

    /// Generate a random referral code (e.g., "ABCD1234")
    pub fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        (0..CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

// TODO: Re-enable these tests once Database referral methods are implemented
#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    // use crate::db::Database;

    #[test]
    #[ignore]
    fn test_generate_referral_code() {
        // let db = Database::new_in_memory().unwrap();
        // let user_id = "user123";

        // let code = db.generate_referral_code(user_id).unwrap();

        // assert_eq!(code.user_id, user_id);
        // assert_eq!(code.code.len(), 8);
        // assert!(code.code.chars().all(|c| c.is_ascii_alphanumeric()));
        // assert!(code.expires_at.is_none());
    }

    #[test]
    fn test_generate_unique_referral_codes() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        let code1 = db.generate_referral_code(user_id).unwrap();
        let code2 = db.generate_referral_code(user_id).unwrap();

        assert_ne!(code1.code, code2.code);
    }

    #[test]
    fn test_validate_referral_code_valid() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        let code = db.generate_referral_code(user_id).unwrap();
        let is_valid = db.validate_referral_code(&code.code).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_validate_referral_code_invalid() {
        let db = Database::new_in_memory().unwrap();

        let is_valid = db.validate_referral_code("INVALID1").unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_validate_referral_code_expired() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        // Create a code that expires immediately
        let expired_time = Utc::now() - chrono::Duration::hours(1);
        let code = db
            .generate_referral_code_with_expiry(user_id, Some(expired_time))
            .unwrap();

        let is_valid = db.validate_referral_code(&code.code).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_apply_referral_code_success() {
        let db = Database::new_in_memory().unwrap();
        let referrer_id = "referrer123";
        let referred_id = "referred456";

        let code = db.generate_referral_code(referrer_id).unwrap();
        let usage = db
            .apply_referral_code(referred_id, &code.code, &RewardConfig::default())
            .unwrap();

        assert_eq!(usage.referred_user_id, referred_id);
        assert_eq!(usage.referrer_user_id, referrer_id);
        assert_eq!(usage.reward_points, 100);
        assert_eq!(usage.reward_credits, Some(50));
        assert_eq!(usage.reward_subscription_days, Some(7));
    }

    #[test]
    fn test_apply_referral_code_self_referral() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        let code = db.generate_referral_code(user_id).unwrap();
        let result = db.apply_referral_code(user_id, &code.code, &RewardConfig::default());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot refer yourself"));
    }

    #[test]
    fn test_apply_referral_code_duplicate() {
        let db = Database::new_in_memory().unwrap();
        let referrer_id = "referrer123";
        let referred_id = "referred456";

        let code = db.generate_referral_code(referrer_id).unwrap();

        // First application should succeed
        db.apply_referral_code(referred_id, &code.code, &RewardConfig::default())
            .unwrap();

        // Second application should fail
        let result = db.apply_referral_code(referred_id, &code.code, &RewardConfig::default());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already used a referral code"));
    }

    #[test]
    fn test_apply_referral_code_invalid() {
        let db = Database::new_in_memory().unwrap();
        let referred_id = "referred456";

        let result = db.apply_referral_code(referred_id, "INVALID1", &RewardConfig::default());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_get_referral_stats_empty() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        let stats = db.get_referral_stats(user_id).unwrap();

        assert_eq!(stats.total_referrals, 0);
        assert_eq!(stats.active_referrals, 0);
        assert_eq!(stats.total_reward_points, 0);
        assert_eq!(stats.total_reward_credits, 0);
        assert_eq!(stats.total_subscription_days, 0);
        assert!(stats.referral_chain.is_empty());
    }

    #[test]
    fn test_get_referral_stats_with_referrals() {
        let db = Database::new_in_memory().unwrap();
        let referrer_id = "referrer123";
        let referred_id1 = "referred1";
        let referred_id2 = "referred2";

        let code = db.generate_referral_code(referrer_id).unwrap();

        db.apply_referral_code(referred_id1, &code.code, &RewardConfig::default())
            .unwrap();
        db.apply_referral_code(referred_id2, &code.code, &RewardConfig::default())
            .unwrap();

        let stats = db.get_referral_stats(referrer_id).unwrap();

        assert_eq!(stats.total_referrals, 2);
        assert_eq!(stats.active_referrals, 2);
        assert_eq!(stats.total_reward_points, 200);
        assert_eq!(stats.total_reward_credits, 100);
        assert_eq!(stats.total_subscription_days, 14);
        assert_eq!(stats.referral_chain.len(), 2);
    }

    #[test]
    fn test_get_referral_chain() {
        let db = Database::new_in_memory().unwrap();
        let user_a = "userA";
        let user_b = "userB";
        let user_c = "userC";

        // A refers B
        let code_a = db.generate_referral_code(user_a).unwrap();
        db.apply_referral_code(user_b, &code_a.code, &RewardConfig::default())
            .unwrap();

        // B refers C
        let code_b = db.generate_referral_code(user_b).unwrap();
        db.apply_referral_code(user_c, &code_b.code, &RewardConfig::default())
            .unwrap();

        // Check B's stats - they referred C
        let stats_b = db.get_referral_stats(user_b).unwrap();
        assert_eq!(stats_b.total_referrals, 1);
        assert!(stats_b.referral_chain.contains(&user_c.to_string()));

        // Check A's stats - they referred B
        let stats_a = db.get_referral_stats(user_a).unwrap();
        assert_eq!(stats_a.total_referrals, 1);
        assert!(stats_a.referral_chain.contains(&user_b.to_string()));
    }

    #[test]
    fn test_list_user_referral_codes() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        db.generate_referral_code(user_id).unwrap();
        db.generate_referral_code(user_id).unwrap();
        db.generate_referral_code(user_id).unwrap();

        let codes = db.list_user_referral_codes(user_id).unwrap();

        assert_eq!(codes.len(), 3);
        assert!(codes.iter().all(|c| c.user_id == user_id));
    }

    #[test]
    fn test_delete_referral_code() {
        let db = Database::new_in_memory().unwrap();
        let user_id = "user123";

        let code = db.generate_referral_code(user_id).unwrap();
        db.delete_referral_code(&code.id).unwrap();

        let is_valid = db.validate_referral_code(&code.code).unwrap();
        assert!(!is_valid);
    }
}
