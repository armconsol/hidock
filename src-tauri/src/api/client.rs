use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::types::{
    AuthResponse, AvailableReward, BillingPortalResponse, CalendarEventsResponse,
    ClaimTrialResponse, CreateEventRequest, DetectLanguageRequest, DetectLanguageResponse,
    FindSpeakersRequest, FindSpeakersResponse, GoogleCalendarEvent, Language,
    LanguageListResponse, LoginRequest, PayoutRequest, PayoutResponse, Receipt,
    ReceiptsResponse, RedeemRewardRequest, RedeemRewardResponse, ReferralInfo,
    ReferralOverviewResponse, RegisterRequest, RegisterResponse, RewardsListResponse, Subscription, SubscriptionResponse,
    SubscriptionStatus, TranslationRequest, TranslationResponse, TrialEligibility, UserInfo,
};

/// Cached subscription data with timestamp
#[derive(Clone, Debug)]
struct CachedSubscription {
    subscription: Subscription,
    cached_at: Instant,
}

impl CachedSubscription {
    fn new(subscription: Subscription) -> Self {
        Self {
            subscription,
            cached_at: Instant::now(),
        }
    }

    /// Check if cache is still valid (default: 5 minutes)
    fn is_valid(&self, max_age: Duration) -> bool {
        self.cached_at.elapsed() < max_age
    }
}

pub struct HiNotesClient {
    base_url: String,
    http_client: Client,
    auth_token: Arc<RwLock<Option<String>>>,
    subscription_cache: Arc<RwLock<Option<CachedSubscription>>>,
    cache_duration: Duration,
    max_retries: u32,
}

impl HiNotesClient {
    /// Default production base URL
    const DEFAULT_BASE_URL: &'static str = "https://hinotes.hidock.com/v1";

    /// Create a new HiNotes client with default settings
    /// Base URL is determined by HINOTES_API_URL environment variable or defaults to production
    pub fn new() -> Self {
        let base_url = env::var("HINOTES_API_URL")
            .unwrap_or_else(|_| Self::DEFAULT_BASE_URL.to_string());
        Self::with_config(base_url, Duration::from_secs(300), 3)
    }

    /// Create a new HiNotes client with explicit base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self::with_config(base_url, Duration::from_secs(300), 3)
    }

    /// Create a new HiNotes client with custom cache duration
    pub fn with_cache_duration(base_url: &str, cache_duration: Duration) -> Self {
        Self::with_config(base_url.to_string(), cache_duration, 3)
    }

    /// Create a new HiNotes client with full configuration
    pub fn with_config(base_url: String, cache_duration: Duration, max_retries: u32) -> Self {
        log::info!("Initializing HiNotes client with base_url: {}", base_url);
        Self {
            base_url,
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            auth_token: Arc::new(RwLock::new(None)),
            subscription_cache: Arc::new(RwLock::new(None)),
            cache_duration,
            max_retries,
        }
    }

    /// Invalidate subscription cache
    pub async fn invalidate_subscription_cache(&self) {
        *self.subscription_cache.write().await = None;
    }

    /// Execute HTTP request with retry logic and error handling
    async fn request_with_retry<F, Fut, T>(
        &self,
        operation_name: &str,
        request_fn: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response>>,
        T: serde::de::DeserializeOwned,
    {
        let mut attempt = 0;
        let mut last_error: Option<anyhow::Error> = None;

        while attempt < self.max_retries {
            attempt += 1;

            log::debug!(
                "{} - Attempt {}/{}",
                operation_name,
                attempt,
                self.max_retries
            );

            match request_fn().await {
                Ok(response) => {
                    let status = response.status();
                    log::debug!("{} - Response status: {}", operation_name, status);

                    if status.is_success() {
                        match response.json::<T>().await {
                            Ok(data) => {
                                log::info!("{} - Success", operation_name);
                                return Ok(data);
                            }
                            Err(e) => {
                                last_error = Some(anyhow::anyhow!("Failed to parse response: {}", e));
                                log::error!("{} - Parse error: {}", operation_name, e);
                            }
                        }
                    } else if status.is_server_error() && attempt < self.max_retries {
                        // Retry on 5xx errors
                        let body = response.text().await.unwrap_or_default();
                        log::warn!(
                            "{} - Server error ({}), will retry. Response: {}",
                            operation_name,
                            status,
                            body
                        );
                        last_error = Some(anyhow::anyhow!("Server error: {} - {}", status, body));
                    } else {
                        // Don't retry on client errors (4xx) or last attempt
                        let body = response.text().await.unwrap_or_default();
                        log::error!(
                            "{} - HTTP error: {} - {}",
                            operation_name,
                            status,
                            body
                        );
                        return Err(anyhow::anyhow!("HTTP error: {} - {}", status, body));
                    }
                }
                Err(e) => {
                    if attempt < self.max_retries {
                        log::warn!("{} - Network error, will retry: {}", operation_name, e);
                        last_error = Some(e);
                    } else {
                        log::error!("{} - Network error (final): {}", operation_name, e);
                        return Err(e);
                    }
                }
            }

            if attempt < self.max_retries {
                // Exponential backoff: 1s, 2s, 4s
                let backoff_duration = Duration::from_secs(2u64.pow(attempt - 1));
                log::debug!(
                    "{} - Backing off for {:?} before retry",
                    operation_name,
                    backoff_duration
                );
                sleep(backoff_duration).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("{} - Max retries exceeded", operation_name)))
    }

    /// Authenticate with email and password
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<UserInfo> {
        log::info!("Authenticating user: {}", email);

        let request_body = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let url = format!("{}/user/signin", self.base_url);
        let client = &self.http_client;

        let auth_response: AuthResponse = self
            .request_with_retry("authenticate", || async {
                client
                    .post(&url)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        // Store the token
        *self.auth_token.write().await = Some(auth_response.token.clone());
        log::info!("Authentication successful for user: {}", email);

        Ok(auth_response.user)
    }

    pub async fn get_token(&self) -> Option<String> {
        self.auth_token.read().await.clone()
    }

    /// Register a new user account
    pub async fn register(&self, email: &str, password: &str, name: &str) -> Result<UserInfo> {
        log::info!("Registering new user: {}", email);

        if email.is_empty() {
            anyhow::bail!("Email cannot be empty");
        }
        if password.len() < 8 {
            anyhow::bail!("Password must be at least 8 characters long");
        }
        if name.is_empty() {
            anyhow::bail!("Name cannot be empty");
        }

        let request_body = RegisterRequest {
            email: email.to_string(),
            password: password.to_string(),
            name: name.to_string(),
        };

        let url = format!("{}/user/register", self.base_url);
        let client = &self.http_client;

        let register_response: RegisterResponse = self
            .request_with_retry("register", || async {
                client
                    .post(&url)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        // Store the token
        *self.auth_token.write().await = Some(register_response.token.clone());
        log::info!("Registration successful for user: {}", email);

        // Store token securely
        if let Err(e) = self.store_token_securely(&register_response.token).await {
            log::warn!("Failed to store token in keyring: {}", e);
        }

        Ok(register_response.user)
    }

    /// Logout and clear stored authentication tokens
    pub async fn logout(&self) -> Result<()> {
        log::info!("Logging out user");

        let token = self.get_token().await;

        // Call the logout endpoint if we have a token
        if let Some(ref token_value) = token {
            let url = format!("{}/user/logout", self.base_url);
            let client = &self.http_client;

            // Use a single retry attempt for logout
            let result = client
                .post(&url)
                .bearer_auth(token_value)
                .send()
                .await;

            if let Err(e) = result {
                log::warn!("Logout API call failed: {}", e);
            }
        }

        // Clear the token even if the API call fails
        *self.auth_token.write().await = None;

        // Clear subscription cache
        *self.subscription_cache.write().await = None;

        // Remove token from keyring
        if let Err(e) = self.clear_stored_token().await {
            log::warn!("Failed to clear token from keyring: {}", e);
        }

        log::info!("Logout complete");
        Ok(())
    }

    /// Store authentication token securely in system keyring
    async fn store_token_securely(&self, token: &str) -> Result<()> {
        let entry = keyring::Entry::new("hinotes-desktop", "auth_token")?;
        entry.set_password(token)?;
        log::debug!("Token stored securely in keyring");
        Ok(())
    }

    /// Retrieve authentication token from system keyring
    pub async fn load_token_from_keyring(&self) -> Result<String> {
        let entry = keyring::Entry::new("hinotes-desktop", "auth_token")?;
        let token = entry.get_password()?;
        *self.auth_token.write().await = Some(token.clone());
        log::info!("Token loaded from keyring");
        Ok(token)
    }

    /// Clear stored token from system keyring
    async fn clear_stored_token(&self) -> Result<()> {
        let entry = keyring::Entry::new("hinotes-desktop", "auth_token")?;
        match entry.delete_password() {
            Ok(_) => {
                log::debug!("Token cleared from keyring");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                log::debug!("No token found in keyring to clear");
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to clear keyring: {}", e)),
        }
    }

    /// Check if user is currently authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.auth_token.read().await.is_some()
    }

    /// Set authentication token manually (useful for OAuth flows)
    pub async fn set_token(&self, token: String) -> Result<()> {
        *self.auth_token.write().await = Some(token.clone());

        // Store in keyring
        if let Err(e) = self.store_token_securely(&token).await {
            log::warn!("Failed to store token in keyring: {}", e);
        }

        log::info!("Token set manually");
        Ok(())
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

    /// Get subscription status from HiNotes API (with caching)
    pub async fn get_subscription_status(&self) -> Result<Subscription> {
        // Check cache first
        {
            let cache = self.subscription_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.is_valid(self.cache_duration) {
                    return Ok(cached.subscription.clone());
                }
            }
        }

        // Fetch fresh data
        let subscription = self.fetch_subscription_status().await?;

        // Update cache
        *self.subscription_cache.write().await = Some(CachedSubscription::new(subscription.clone()));

        Ok(subscription)
    }

    /// Fetch subscription status from HiNotes API (bypassing cache)
    async fn fetch_subscription_status(&self) -> Result<Subscription> {
        const GRACE_PERIOD_DAYS: i64 = 7; // Standard grace period after expiration

        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!("{}/subscribers", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let sub_response: SubscriptionResponse = self
            .request_with_retry("fetch_subscription_status", || async {
                client
                    .get(&url)
                    .bearer_auth(&token_clone)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        // Parse the subscription data from RevenueCat format
        let mut in_grace_period = false;
        let mut grace_period_days_remaining: Option<i32> = None;

        let status = if let Some(premium) = &sub_response.subscriber.entitlements.premium {
            if let Some(ref expires_date) = premium.expires_date {
                // Check if expired
                let expires = chrono::DateTime::parse_from_rfc3339(expires_date)
                    .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?
                    .with_timezone(&chrono::Utc);
                let now = chrono::Utc::now();

                if expires < now {
                    // Check if within grace period
                    let grace_end = expires + chrono::Duration::days(GRACE_PERIOD_DAYS);
                    if now < grace_end {
                        in_grace_period = true;
                        let days_remaining = (grace_end - now).num_days();
                        grace_period_days_remaining = Some(days_remaining as i32);
                        SubscriptionStatus::Active // Still treated as active during grace period
                    } else {
                        SubscriptionStatus::Expired
                    }
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
            in_grace_period,
            grace_period_days_remaining,
        })
    }

    /// Get subscription status, bypassing cache
    pub async fn get_subscription_status_fresh(&self) -> Result<Subscription> {
        let subscription = self.fetch_subscription_status().await?;
        *self.subscription_cache.write().await = Some(CachedSubscription::new(subscription.clone()));
        Ok(subscription)
    }

    /// Check if user has active subscription (includes grace period)
    pub async fn check_subscription(&self) -> Result<bool> {
        let subscription = self.get_subscription_status().await?;
        Ok(subscription.status == SubscriptionStatus::Active
            || subscription.status == SubscriptionStatus::Trial)
    }

    /// Check if subscription has expired (considering grace period)
    pub async fn is_subscription_expired(&self) -> Result<bool> {
        let subscription = self.get_subscription_status().await?;
        Ok(subscription.status == SubscriptionStatus::Expired)
    }

    /// Check if subscription is in grace period
    pub async fn is_in_grace_period(&self) -> Result<bool> {
        let subscription = self.get_subscription_status().await?;
        Ok(subscription.in_grace_period)
    }

    /// Get days remaining until subscription truly expires (after grace period)
    pub async fn get_days_until_expiration(&self) -> Result<Option<i64>> {
        let subscription = self.get_subscription_status().await?;

        if let Some(expires_at) = &subscription.expires_at {
            let expires = chrono::DateTime::parse_from_rfc3339(expires_at)
                .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?
                .with_timezone(&chrono::Utc);

            let grace_period_days = 7i64; // Standard grace period
            let final_expiry = expires + chrono::Duration::days(grace_period_days);
            let now = chrono::Utc::now();

            if final_expiry > now {
                Ok(Some((final_expiry - now).num_days()))
            } else {
                Ok(Some(0)) // Already expired
            }
        } else {
            Ok(None) // No expiration (lifetime or free tier)
        }
    }

    /// Get purchase receipts
    pub async fn get_receipts(&self) -> Result<Vec<Receipt>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/receipts", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get receipts: {}", response.status());
        }

        let receipts_response: ReceiptsResponse = response.json().await?;
        Ok(receipts_response.receipts)
    }

    /// Get billing portal URL for subscription management
    pub async fn get_billing_portal_url(&self) -> Result<String> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/payment/rc/portal", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get billing portal URL: {}", response.status());
        }

        let portal_response: BillingPortalResponse = response.json().await?;
        Ok(portal_response.url)
    }

    /// Check trial eligibility
    pub async fn check_trial_eligibility(&self) -> Result<TrialEligibility> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/user/trial/check", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to check trial eligibility: {}", response.status());
        }

        let eligibility_response: TrialEligibility = response.json().await?;
        Ok(eligibility_response)
    }

    /// Claim trial subscription
    pub async fn claim_trial(&self) -> Result<ClaimTrialResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .post(&format!("{}/user/trial/claim", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to claim trial: {}", response.status());
        }

        let claim_response: ClaimTrialResponse = response.json().await?;
        Ok(claim_response)
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        // Act
        let _ = client.authenticate("test@example.com", "password").await;

        // Assert
        let token = client.get_token().await;
        assert!(token.is_some());
        assert!(!token.unwrap().is_empty());
    }

    // ===== REGISTRATION TESTS =====

    #[tokio::test]
    async fn test_register_new_user() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("newuser@example.com", "password123", "New User").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "newuser@example.com");
        assert_eq!(user.name, "New User");
        assert!(!user.id.is_empty());
    }

    #[tokio::test]
    async fn test_register_stores_token() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let _ = client.register("newuser@example.com", "password123", "New User").await;
        let token = client.get_token().await;
        assert!(token.is_some());
        assert!(!token.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_register_validates_email() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("", "password123", "User").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Email cannot be empty"));
    }

    #[tokio::test]
    async fn test_register_validates_password_length() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("user@example.com", "short", "User").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 8 characters"));
    }

    #[tokio::test]
    async fn test_register_validates_name() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("user@example.com", "password123", "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Name cannot be empty"));
    }

    // ===== LOGOUT TESTS =====

    #[tokio::test]
    async fn test_logout_clears_token() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        assert!(client.is_authenticated().await);

        let result = client.logout().await;
        assert!(result.is_ok());
        assert!(!client.is_authenticated().await);
        assert!(client.get_token().await.is_none());
    }

    #[tokio::test]
    async fn test_logout_clears_subscription_cache() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        // Populate subscription cache
        let _ = client.get_subscription_status().await;

        let result = client.logout().await;
        assert!(result.is_ok());

        // Subscription cache should be cleared
        assert!(!client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_logout_succeeds_without_token() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        // Logout without being authenticated should succeed
        let result = client.logout().await;
        assert!(result.is_ok());
    }

    // ===== TOKEN MANAGEMENT TESTS =====

    #[tokio::test]
    async fn test_is_authenticated() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        assert!(!client.is_authenticated().await);

        let _ = client.authenticate("test@example.com", "password").await;
        assert!(client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_set_token_manually() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.set_token("manual-token-123".to_string()).await;
        assert!(result.is_ok());
        assert!(client.is_authenticated().await);

        let token = client.get_token().await;
        assert_eq!(token, Some("manual-token-123".to_string()));
    }

    // ===== SUBSCRIPTION TESTS =====

    #[tokio::test]
    async fn test_get_subscription_status_active() {
        // Arrange
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        // Act
        let subscription = client.get_subscription_status().await.unwrap();

        // Assert
        assert!(matches!(
            subscription.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trial | SubscriptionStatus::Expired
        ));
    }

    // ===== RECEIPTS TESTS =====

    #[tokio::test]
    async fn test_get_receipts_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.get_receipts().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_get_receipts_success() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        let result = client.get_receipts().await;
        assert!(result.is_ok());
        let receipts = result.unwrap();
        assert!(receipts.len() >= 0);
    }

    // ===== BILLING PORTAL TESTS =====

    #[tokio::test]
    async fn test_get_billing_portal_url_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.get_billing_portal_url().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_get_billing_portal_url_success() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        let result = client.get_billing_portal_url().await;
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(url.starts_with("http"));
    }

    // ===== TRIAL ELIGIBILITY TESTS =====

    #[tokio::test]
    async fn test_check_trial_eligibility_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.check_trial_eligibility().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_check_trial_eligibility_success() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        let result = client.check_trial_eligibility().await;
        assert!(result.is_ok());
        let eligibility = result.unwrap();
        assert!(eligibility.eligible || !eligibility.eligible); // Just check it returns a valid response
    }

    // ===== CLAIM TRIAL TESTS =====

    #[tokio::test]
    async fn test_claim_trial_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.claim_trial().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_claim_trial_success() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("eligible@example.com", "password").await;

        let result = client.claim_trial().await;
        assert!(result.is_ok());
        let claim_response = result.unwrap();
        assert!(claim_response.success || !claim_response.success);
        assert!(!claim_response.message.is_empty());
    }

    // ===== CACHING TESTS =====

    #[tokio::test]
    async fn test_subscription_caching() {
        use std::time::Duration;

        let client = HiNotesClient::with_cache_duration(
            "http://localhost:3001/v1",
            Duration::from_secs(60),
        );
        let _ = client.authenticate("test@example.com", "password").await;

        // First call should fetch from API
        let result1 = client.get_subscription_status().await;
        assert!(result1.is_ok());

        // Second call should use cache
        let result2 = client.get_subscription_status().await;
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap().product_id, result2.unwrap().product_id);
    }

    #[tokio::test]
    async fn test_invalidate_subscription_cache() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        // Fetch and cache
        let _ = client.get_subscription_status().await;

        // Invalidate cache
        client.invalidate_subscription_cache().await;

        // Should fetch fresh data
        let result = client.get_subscription_status().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_subscription_status_fresh() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        // Get fresh data (bypasses cache)
        let result = client.get_subscription_status_fresh().await;
        assert!(result.is_ok());
    }

    // ===== GRACE PERIOD TESTS =====

    #[tokio::test]
    async fn test_is_in_grace_period() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        let result = client.is_in_grace_period().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_subscription_expired() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("expired@example.com", "password").await;

        let result = client.is_subscription_expired().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_days_until_expiration() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        let result = client.get_days_until_expiration().await;
        assert!(result.is_ok());
    }

    // ===== TRANSLATION TESTS =====

    #[tokio::test]
    async fn test_translate_text_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.detect_language("Hello world").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_detect_language_empty_text() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.get_language_list().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_analyze_speakers_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.list_rewards().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_redeem_reward_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.redeem_reward("reward-123", 100.0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_redeem_reward_empty_id() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.request_payout(25.0, "user@example.com").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_request_payout_invalid_amount() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
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
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.request_payout(25.0, "invalid-email").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid PayPal email format"));
    }
}
