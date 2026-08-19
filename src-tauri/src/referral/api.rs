use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Referral API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralOverview {
    pub referral_code: String,
    pub total_rewards: f64,
    pub total_referrals: i32,
    pub pending_rewards: f64,
    pub available_cash: Option<f64>,
    pub available_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsOverview {
    pub total_points: f64,
    pub available_rewards: Vec<RewardOption>,
    pub redemption_history: Vec<RedemptionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardOption {
    pub reward_type: RewardType,
    pub amount: f64,
    pub points_required: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RewardType {
    Minutes,
    Cash,
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedemptionRecord {
    pub id: String,
    pub reward_type: RewardType,
    pub amount: f64,
    pub redeemed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChooseMinutesRequest {
    pub minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChooseMinutesResponse {
    pub success: bool,
    pub minutes_granted: i32,
    pub points_used: f64,
    pub remaining_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub subject: String,
    pub body: String,
    pub share_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPalConnectRequest {
    pub paypal_email: String,
    pub authorization_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPalConnectResponse {
    pub success: bool,
    pub paypal_email: String,
    pub connected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPalDisconnectResponse {
    pub success: bool,
    pub message: String,
}

/// API client for referral operations
pub struct ReferralApiClient {
    base_url: String,
    http_client: Client,
    auth_token: Arc<RwLock<Option<String>>>,
}

impl ReferralApiClient {
    /// Create a new referral API client
    pub fn new(base_url: String, auth_token: Arc<RwLock<Option<String>>>) -> Self {
        Self {
            base_url,
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            auth_token,
        }
    }

    /// Get authorization header value
    async fn get_auth_header(&self) -> Result<String> {
        let token = self.auth_token.read().await;
        token
            .as_ref()
            .map(|t| format!("Bearer {}", t))
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))
    }

    /// GET /v1/referral/overview
    /// Get referral program details
    pub async fn get_referral_overview(&self) -> Result<ReferralOverview> {
        let url = format!("{}/referral/overview", self.base_url);
        let auth = self.get_auth_header().await?;

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        Ok(response.json().await?)
    }

    /// GET /v1/referral/rewards-overview
    /// Get user's referral rewards
    pub async fn get_rewards_overview(&self) -> Result<RewardsOverview> {
        let url = format!("{}/referral/rewards-overview", self.base_url);
        let auth = self.get_auth_header().await?;

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        Ok(response.json().await?)
    }

    /// POST /v1/referral/choose-minutes
    /// Claim transcription minutes as reward
    pub async fn choose_minutes(&self, minutes: i32) -> Result<ChooseMinutesResponse> {
        let url = format!("{}/referral/choose-minutes", self.base_url);
        let auth = self.get_auth_header().await?;

        let request_body = ChooseMinutesRequest { minutes };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", auth)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        Ok(response.json().await?)
    }

    /// GET /v1/referral/message-template
    /// Get referral message templates
    pub async fn get_message_template(&self) -> Result<MessageTemplate> {
        let url = format!("{}/referral/message-template", self.base_url);
        let auth = self.get_auth_header().await?;

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        Ok(response.json().await?)
    }

    /// POST /v1/referral/paypal/connect
    /// Connect PayPal for referral payouts
    pub async fn connect_paypal(
        &self,
        paypal_email: &str,
        authorization_code: Option<String>,
    ) -> Result<PayPalConnectResponse> {
        let url = format!("{}/referral/paypal/connect", self.base_url);
        let auth = self.get_auth_header().await?;

        let request_body = PayPalConnectRequest {
            paypal_email: paypal_email.to_string(),
            authorization_code,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", auth)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        Ok(response.json().await?)
    }

    /// POST /v1/referral/paypal/disconnect
    /// Disconnect PayPal account
    pub async fn disconnect_paypal(&self) -> Result<PayPalDisconnectResponse> {
        let url = format!("{}/referral/paypal/disconnect", self.base_url);
        let auth = self.get_auth_header().await?;

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        Ok(response.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;

    #[test]
    fn test_client_creation() {
        let token = Arc::new(RwLock::new(Some("test-token".to_string())));
        let client = ReferralApiClient::new("https://api.example.com/v1".to_string(), token);
        assert_eq!(client.base_url, "https://api.example.com/v1");
    }

    #[tokio::test]
    async fn test_auth_header() {
        let token = Arc::new(RwLock::new(Some("test-token".to_string())));
        let client = ReferralApiClient::new("https://api.example.com/v1".to_string(), token);

        let auth_header = client.get_auth_header().await.unwrap();
        assert_eq!(auth_header, "Bearer test-token");
    }

    #[tokio::test]
    async fn test_auth_header_no_token() {
        let token = Arc::new(RwLock::new(None));
        let client = ReferralApiClient::new("https://api.example.com/v1".to_string(), token);

        let result = client.get_auth_header().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not authenticated"));
    }
}
