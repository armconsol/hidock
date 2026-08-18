use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{
    AuthResponse, AvailableReward, CalendarEventsResponse, CreateEventRequest,
    DetectLanguageRequest, DetectLanguageResponse, FindSpeakersRequest, FindSpeakersResponse,
    GoogleCalendarEvent, Language, LanguageListResponse, LoginRequest, PayoutRequest,
    PayoutResponse, RedeemRewardRequest, RedeemRewardResponse, ReferralInfo,
    ReferralOverviewResponse, RewardsListResponse, Subscription, SubscriptionResponse,
    SubscriptionStatus, TranslationRequest, TranslationResponse, UserInfo,
};

pub struct HiNotesClient {
    base_url: String,
    http_client: Client,
    auth_token: Arc<RwLock<Option<String>>>,
}

impl HiNotesClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: Client::new(),
            auth_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Authenticate with email and password
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<UserInfo> {
        let request_body = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .http_client
            .post(&format!("{}/user/signin", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        let auth_response: AuthResponse = response.json().await?;

        // Store the token
        *self.auth_token.write().await = Some(auth_response.token);

        Ok(auth_response.user)
    }

    pub async fn get_token(&self) -> Option<String> {
        self.auth_token.read().await.clone()
    }

    /// List calendar events from Google Calendar
    pub async fn list_events(
        &self,
        calendar_id: &str,
        time_min: DateTime<Utc>,
        time_max: DateTime<Utc>,
    ) -> Result<Vec<GoogleCalendarEvent>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            calendar_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .query(&[
                ("timeMin", time_min.to_rfc3339()),
                ("timeMax", time_max.to_rfc3339()),
                ("singleEvents", "true".to_string()),
                ("orderBy", "startTime".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch calendar events: {}", response.status());
        }

        let calendar_response: CalendarEventsResponse = response.json().await?;
        Ok(calendar_response.items)
    }

    /// Add a new calendar event to Google Calendar
    pub async fn add_event(
        &self,
        calendar_id: &str,
        event: CreateEventRequest,
    ) -> Result<GoogleCalendarEvent> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            calendar_id
        );

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&event)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create calendar event: {}", response.status());
        }

        let created_event: GoogleCalendarEvent = response.json().await?;
        Ok(created_event)
    }

    /// Get subscription status from HiNotes API
    pub async fn get_subscription_status(&self) -> Result<Subscription> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/subscribers", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get subscription status: {}", response.status());
        }

        let sub_response: SubscriptionResponse = response.json().await?;

        // Parse the subscription data from RevenueCat format
        let status = if let Some(premium) = &sub_response.subscriber.entitlements.premium {
            if let Some(ref expires_date) = premium.expires_date {
                // Check if expired
                let expires = chrono::DateTime::parse_from_rfc3339(expires_date)
                    .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?;
                if expires < chrono::Utc::now() {
                    SubscriptionStatus::Expired
                } else {
                    SubscriptionStatus::Active
                }
            } else {
                SubscriptionStatus::Active
            }
        } else {
            SubscriptionStatus::Expired
        };

        let product_id = sub_response
            .subscriber
            .entitlements
            .premium
            .as_ref()
            .map(|e| e.product_identifier.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let expires_at = sub_response
            .subscriber
            .entitlements
            .premium
            .and_then(|e| e.expires_date);

        Ok(Subscription {
            product_id,
            status,
            expires_at,
        })
    }

    /// Check if user has active subscription
    pub async fn check_subscription(&self) -> Result<bool> {
        let subscription = self.get_subscription_status().await?;
        Ok(subscription.status == SubscriptionStatus::Active
            || subscription.status == SubscriptionStatus::Trial)
    }

    /// Get referral information
    pub async fn get_referral_info(&self) -> Result<ReferralInfo> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/referral/rewards-overview", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get referral info: {}", response.status());
        }

        let referral_response: ReferralOverviewResponse = response.json().await?;

        Ok(ReferralInfo {
            code: referral_response.referral_code,
            rewards_earned: referral_response.total_rewards,
            referrals_count: referral_response.total_referrals,
        })
    }

    /// Translate text from source language to target language
    pub async fn translate_text(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if request.text.is_empty() {
            anyhow::bail!("Translation text cannot be empty");
        }

        let response = self
            .http_client
            .post(&format!("{}/translate", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to translate text: {}", response.status());
        }

        let translation_response: TranslationResponse = response.json().await?;
        Ok(translation_response)
    }

    /// Detect the language of the given text
    pub async fn detect_language(&self, text: &str) -> Result<String> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if text.is_empty() {
            anyhow::bail!("Text for language detection cannot be empty");
        }

        let request = DetectLanguageRequest {
            text: text.to_string(),
        };

        let response = self
            .http_client
            .post(&format!("{}/detect-language", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to detect language: {}", response.status());
        }

        let detect_response: DetectLanguageResponse = response.json().await?;
        Ok(detect_response.detected_lang)
    }

    /// Get list of supported languages for translation
    pub async fn get_language_list(&self) -> Result<Vec<Language>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/live/language/list", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get language list: {}", response.status());
        }

        let lang_response: LanguageListResponse = response.json().await?;
        Ok(lang_response.languages)
    }

    /// Analyze speakers in a note's audio recording
    pub async fn analyze_speakers(
        &self,
        request: FindSpeakersRequest,
    ) -> Result<FindSpeakersResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if request.note_id.is_empty() {
            anyhow::bail!("Note ID cannot be empty");
        }

        let response = self
            .http_client
            .post(&format!("{}/note/speaker/find", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to analyze speakers: {}", response.status());
        }

        let speaker_response: FindSpeakersResponse = response.json().await?;
        Ok(speaker_response)
    }

    /// List available rewards
    pub async fn list_rewards(&self) -> Result<Vec<AvailableReward>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/redemption/info", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get rewards list: {}", response.status());
        }

        let rewards_response: RewardsListResponse = response.json().await?;
        Ok(rewards_response.rewards)
    }

    /// Redeem a reward by ID
    pub async fn redeem_reward(
        &self,
        reward_id: &str,
        points: f64,
    ) -> Result<RedeemRewardResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if reward_id.is_empty() {
            anyhow::bail!("Reward ID cannot be empty");
        }

        if points <= 0.0 {
            anyhow::bail!("Points must be greater than zero");
        }

        let request = RedeemRewardRequest {
            reward_id: reward_id.to_string(),
            points,
        };

        let response = self
            .http_client
            .post(&format!("{}/redemption/fulfill", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to redeem reward: {}", response.status());
        }

        let redeem_response: RedeemRewardResponse = response.json().await?;

        if !redeem_response.success {
            anyhow::bail!("Reward redemption failed: {}", redeem_response.message);
        }

        Ok(redeem_response)
    }

    /// Request a PayPal payout
    pub async fn request_payout(&self, amount: f64, paypal_email: &str) -> Result<PayoutResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if amount <= 0.0 {
            anyhow::bail!("Payout amount must be greater than zero");
        }

        if paypal_email.is_empty() {
            anyhow::bail!("PayPal email is required");
        }

        if !paypal_email.contains('@') {
            anyhow::bail!("Invalid PayPal email format");
        }

        let request = PayoutRequest {
            amount,
            paypal_email: paypal_email.to_string(),
        };

        let response = self
            .http_client
            .post(&format!("{}/referral/paypal/payout", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to request payout: {}", response.status());
        }

        let payout_response: PayoutResponse = response.json().await?;
        Ok(payout_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticate_with_credentials() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");

        // Act
        let result = client.authenticate("test@example.com", "password").await;

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert!(!user.id.is_empty());
        assert!(!user.name.is_empty());
    }

    #[tokio::test]
    async fn test_token_is_stored_after_auth() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");

        // Act
        let _ = client.authenticate("test@example.com", "password").await;

        // Assert
        let token = client.get_token().await;
        assert!(token.is_some());
        assert!(!token.unwrap().is_empty());
    }

    // ===== SUBSCRIPTION TESTS =====

    #[tokio::test]
    async fn test_get_subscription_status_active() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");
        let _ = client.authenticate("test@example.com", "password").await;

        // Act
        let result = client.get_subscription_status().await;

        // Assert
        assert!(result.is_ok());
        let subscription = result.unwrap();
        assert_eq!(subscription.status, SubscriptionStatus::Active);
        assert!(!subscription.product_id.is_empty());
    }

    #[tokio::test]
    async fn test_get_subscription_status_not_authenticated() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");

        // Act
        let result = client.get_subscription_status().await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_check_subscription_returns_true_for_active() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");
        let _ = client.authenticate("test@example.com", "password").await;

        // Act
        let result = client.check_subscription().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_check_subscription_returns_false_for_expired() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");
        let _ = client.authenticate("expired@example.com", "password").await;

        // Act
        let result = client.check_subscription().await;

        // Assert
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_get_referral_info() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");
        let _ = client.authenticate("test@example.com", "password").await;

        // Act
        let result = client.get_referral_info().await;

        // Assert
        assert!(result.is_ok());
        let referral_info = result.unwrap();
        assert!(!referral_info.code.is_empty());
        assert!(referral_info.rewards_earned >= 0.0);
        assert!(referral_info.referrals_count >= 0);
    }

    #[tokio::test]
    async fn test_get_referral_info_not_authenticated() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");

        // Act
        let result = client.get_referral_info().await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_subscription_status_parsing_from_response() {
        // Arrange
        let client = HiNotesClient::new("http://localhost:3001/v1");
        let _ = client.authenticate("test@example.com", "password").await;

        // Act
        let subscription = client.get_subscription_status().await.unwrap();

        // Assert
        assert!(matches!(
            subscription.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trial | SubscriptionStatus::Expired
        ));
    }

    // ===== TRANSLATION TESTS =====

    #[tokio::test]
    async fn test_translate_text_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let request = TranslationRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };

        let result = client.translate_text(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_translate_text_empty_text() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let request = TranslationRequest {
            text: "".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };

        let result = client.translate_text(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Translation text cannot be empty"));
    }

    #[tokio::test]
    async fn test_detect_language_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let result = client.detect_language("Hello world").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_detect_language_empty_text() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.detect_language("").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Text for language detection cannot be empty"));
    }

    #[tokio::test]
    async fn test_get_language_list_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let result = client.get_language_list().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_analyze_speakers_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let request = FindSpeakersRequest {
            note_id: "note-123".to_string(),
            audio_url: Some("https://example.com/audio.mp3".to_string()),
        };

        let result = client.analyze_speakers(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_analyze_speakers_empty_note_id() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let request = FindSpeakersRequest {
            note_id: "".to_string(),
            audio_url: None,
        };

        let result = client.analyze_speakers(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Note ID cannot be empty"));
    }

    // ===== REWARD REDEMPTION TESTS =====

    #[tokio::test]
    async fn test_list_rewards_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let result = client.list_rewards().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_redeem_reward_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let result = client.redeem_reward("reward-123", 100.0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_redeem_reward_empty_id() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.redeem_reward("", 100.0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Reward ID cannot be empty"));
    }

    #[tokio::test]
    async fn test_redeem_reward_invalid_points() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.redeem_reward("reward-123", 0.0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Points must be greater than zero"));
    }

    #[tokio::test]
    async fn test_redeem_reward_negative_points() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.redeem_reward("reward-123", -10.0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Points must be greater than zero"));
    }

    #[tokio::test]
    async fn test_request_payout_requires_auth() {
        let client = HiNotesClient::new("http://localhost:3001/v1");

        let result = client.request_payout(25.0, "user@example.com").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_request_payout_invalid_amount() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.request_payout(0.0, "user@example.com").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Payout amount must be greater than zero"));
    }

    #[tokio::test]
    async fn test_request_payout_empty_email() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.request_payout(25.0, "").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("PayPal email is required"));
    }

    #[tokio::test]
    async fn test_request_payout_invalid_email() {
        let client = HiNotesClient::new("http://localhost:3001/v1");
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.request_payout(25.0, "invalid-email").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid PayPal email format"));
    }
}
