use hinotes_desktop_lib::db::Database;
use hinotes_desktop_lib::referral::RewardConfig;

#[test]
fn test_referral_system_full_flow() {
    let db = Database::new_in_memory().unwrap();

    // 1. Generate referral code
    let referrer_id = "user_alice";
    let code = db.generate_referral_code(referrer_id).unwrap();
    println!("Generated code: {}", code.code);
    assert_eq!(code.user_id, referrer_id);
    assert_eq!(code.code.len(), 8);

    // 2. Validate the code
    let is_valid = db.validate_referral_code(&code.code).unwrap();
    assert!(is_valid);

    // 3. Apply referral code
    let referred_id = "user_bob";
    let usage = db
        .apply_referral_code(referred_id, &code.code, &RewardConfig::default())
        .unwrap();

    assert_eq!(usage.referred_user_id, referred_id);
    assert_eq!(usage.referrer_user_id, referrer_id);
    assert_eq!(usage.reward_points, 100);

    // 4. Get stats
    let stats = db.get_referral_stats(referrer_id).unwrap();
    assert_eq!(stats.total_referrals, 1);
    assert_eq!(stats.total_reward_points, 100);
    assert_eq!(stats.referral_chain.len(), 1);

    println!("✓ Full referral flow test passed");
}

#[test]
fn test_self_referral_prevention() {
    let db = Database::new_in_memory().unwrap();

    let user_id = "user123";
    let code = db.generate_referral_code(user_id).unwrap();

    // Try to refer yourself
    let result = db.apply_referral_code(user_id, &code.code, &RewardConfig::default());

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot refer yourself"));

    println!("✓ Self-referral prevention test passed");
}

#[test]
fn test_duplicate_referral_prevention() {
    let db = Database::new_in_memory().unwrap();

    let referrer_id = "referrer";
    let referred_id = "referred";

    let code = db.generate_referral_code(referrer_id).unwrap();

    // First application should succeed
    db.apply_referral_code(referred_id, &code.code, &RewardConfig::default())
        .unwrap();

    // Second application should fail
    let result = db.apply_referral_code(referred_id, &code.code, &RewardConfig::default());

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already used"));

    println!("✓ Duplicate referral prevention test passed");
}
