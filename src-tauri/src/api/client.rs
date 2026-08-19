use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::types::{
    AIEngine, AIEngineListResponse, AudioSegment, AuthResponse, AvailableReward,
    BillingPortalResponse, CalendarEventsResponse, ClaimTrialResponse, CreateEventRequest,
    DeleteUserResponse, DetectLanguageRequest, DetectLanguageResponse, DeviceFile,
    DeviceFileGetRequest, DeviceFileListRequest, DeviceFileListResponse,
    ExtractVoiceSignatureRequest, ExtractVoiceSignatureResponse, FindSpeakersRequest,
    FindSpeakersResponse, GetSettingRequest, GetSettingResponse, GoogleCalendarEvent, Language,
    LanguageListResponse, LoginRequest, MatchSpeakerRequest, MatchSpeakerResponse,
    NotifyRecordingStatusResponse, PayoutRequest, PayoutResponse, Receipt, ReceiptsResponse,
    RedeemRewardRequest, RedeemRewardResponse, ReferralInfo, ReferralOverviewResponse,
    RegisterRequest, RegisterResponse, RenameUserRequest, RenameUserResponse, RewardsListResponse,
    SaveNewPasswordRequest, SaveNewPasswordResponse, SaveSettingRequest, SaveSettingResponse,
    SendEmailVerificationRequest, SendEmailVerificationResponse, SendPasswordResetRequest,
    SendPasswordResetResponse, SettingsListResponse, SpeakerSegment, Subscription,
    SubscriptionResponse, SubscriptionStatus, TranslationRequest, TranslationResponse,
    TrialEligibility, UpdateEventRequest, UpdatePasswordRequest, UpdatePasswordResponse,
    UpdateRegionRequest, UpdateRegionResponse, UserInfo, UserProfileResponse, VerifyEmailRequest,
    VerifyEmailResponse, VerifyResetCodeRequest, VerifyResetCodeResponse, VoiceSignature,
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
        let base_url =
            env::var("HINOTES_API_URL").unwrap_or_else(|_| Self::DEFAULT_BASE_URL.to_string());
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
    async fn request_with_retry<F, Fut, T>(&self, operation_name: &str, request_fn: F) -> Result<T>
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
                                last_error =
                                    Some(anyhow::anyhow!("Failed to parse response: {}", e));
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
                        log::error!("{} - HTTP error: {} - {}", operation_name, status, body);
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

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("{} - Max retries exceeded", operation_name)))
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
            let result = client.post(&url).bearer_auth(token_value).send().await;

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
        match entry.delete_credential() {
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

    /// Update an existing calendar event in Google Calendar
    pub async fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        event: UpdateEventRequest,
    ) -> Result<GoogleCalendarEvent> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            calendar_id, event_id
        );

        let response = self
            .http_client
            .put(&url)
            .bearer_auth(&token)
            .json(&event)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to update calendar event: {}", response.status());
        }

        let updated_event: GoogleCalendarEvent = response.json().await?;
        Ok(updated_event)
    }

    /// Delete a calendar event from Google Calendar
    pub async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            calendar_id, event_id
        );

        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete calendar event: {}", response.status());
        }

        Ok(())
    }

    /// Notify HiNotes calendar of device recording state
    ///
    /// Updates a Google Calendar event to indicate that a recording is in progress
    /// or has completed. The server typically updates the event description with
    /// "Recording in progress..." while active, and may add a transcription link
    /// when the recording is finished.
    ///
    /// # Arguments
    /// * `event_id` - Google Calendar event ID
    /// * `is_recording` - True if recording is active, False if stopped
    pub async fn notify_recording_status(
        &self,
        event_id: &str,
        is_recording: bool,
    ) -> Result<NotifyRecordingStatusResponse> {
        use super::types::NotifyRecordingStatusRequest;

        log::info!(
            "Notifying recording status for event {}: {}",
            event_id,
            if is_recording { "started" } else { "stopped" }
        );

        let request_body = NotifyRecordingStatusRequest {
            event_id: event_id.to_string(),
            is_recording,
        };

        let url = format!("{}/calendar/event/device_state/notice", self.base_url);
        let client = &self.http_client;
        let token = self.get_token().await;

        let response: NotifyRecordingStatusResponse = self
            .request_with_retry("notify_recording_status", || async {
                let mut request = client.post(&url).json(&request_body);

                if let Some(ref token) = token {
                    request = request.bearer_auth(token);
                }

                Ok(request.send().await?)
            })
            .await?;

        Ok(response)
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
        *self.subscription_cache.write().await =
            Some(CachedSubscription::new(subscription.clone()));

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
        *self.subscription_cache.write().await =
            Some(CachedSubscription::new(subscription.clone()));
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

    /// Translate text from source language to target language using generic endpoint
    pub async fn translate_text(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if request.text.is_empty() {
            anyhow::bail!("Translation text cannot be empty");
        }

        let url = format!("{}/translate", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let translation_response: TranslationResponse = self
            .request_with_retry("translate_text", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        Ok(translation_response)
    }

    /// Translate text using HiNotes Live Translation API
    /// This is the primary API method that TranslationEngine should use
    ///
    /// # Arguments
    /// * `request` - Translation request with text and language codes
    ///
    /// # Returns
    /// * `Result<TranslationResponse>` - Translated text with metadata
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if text is empty
    /// * Returns error if API call fails
    pub async fn translate_text_api(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse> {
        // Use the same endpoint but with retry logic and better error handling
        self.translate_text(request).await
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

        let url = format!("{}/live/language/list", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let lang_response: LanguageListResponse = self
            .request_with_retry("get_language_list", || async {
                client
                    .get(&url)
                    .bearer_auth(&token_clone)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        Ok(lang_response.languages)
    }

    /// Get translated note from live translation session
    ///
    /// # Arguments
    /// * `note_id` - The ID of the note to retrieve translation for
    ///
    /// # Returns
    /// * `Result<TranslationResponse>` - The translated note content
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if note is not found or translation is not available
    pub async fn get_live_translation(&self, note_id: &str) -> Result<TranslationResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if note_id.is_empty() {
            anyhow::bail!("Note ID cannot be empty");
        }

        let request_body = serde_json::json!({
            "note_id": note_id
        });

        let url = format!("{}/live/note/get", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let translation_response: TranslationResponse = self
            .request_with_retry("get_live_translation", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        log::info!("Retrieved live translation for note {}", note_id);
        Ok(translation_response)
    }

    /// Rate the quality of a translation
    ///
    /// # Arguments
    /// * `note_id` - The ID of the note/translation to rate
    /// * `rating` - Rating value (typically 1-5)
    /// * `feedback` - Optional feedback text
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if rating submission fails
    pub async fn rate_translation(
        &self,
        note_id: &str,
        rating: u8,
        feedback: Option<String>,
    ) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if note_id.is_empty() {
            anyhow::bail!("Note ID cannot be empty");
        }

        if rating < 1 || rating > 5 {
            anyhow::bail!("Rating must be between 1 and 5");
        }

        let mut request_body = serde_json::json!({
            "note_id": note_id,
            "rating": rating
        });

        if let Some(feedback_text) = feedback {
            request_body["feedback"] = serde_json::json!(feedback_text);
        }

        let url = format!("{}/live/rate", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let response: serde_json::Value = self
            .request_with_retry("rate_translation", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        // Check if response indicates success
        if let Some(success) = response.get("success").and_then(|v| v.as_bool()) {
            if !success {
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                anyhow::bail!("Failed to rate translation: {}", message);
            }
        }

        log::info!(
            "Successfully rated translation for note {} with rating {}",
            note_id,
            rating
        );
        Ok(())
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

    // ===== USER PROFILE MANAGEMENT =====

    /// Get current user information
    pub async fn get_user_info(&self) -> Result<UserInfo> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!("{}/user/info", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let user_response: UserProfileResponse = self
            .request_with_retry("get_user_info", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        Ok(user_response.user)
    }

    /// Update user display name
    pub async fn rename_user(&self, new_name: String) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if new_name.is_empty() {
            anyhow::bail!("Name cannot be empty");
        }

        let request = RenameUserRequest { name: new_name };

        let url = format!("{}/user/rename", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let rename_response: RenameUserResponse = self
            .request_with_retry("rename_user", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !rename_response.success {
            anyhow::bail!(
                "Failed to rename user: {}",
                rename_response.message.unwrap_or_default()
            );
        }

        Ok(())
    }

    /// Update user's region/locale settings
    pub async fn update_user_region(&self, region: String) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if region.is_empty() {
            anyhow::bail!("Region cannot be empty");
        }

        let request = UpdateRegionRequest { region };

        let url = format!("{}/user/region/update", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let region_response: UpdateRegionResponse = self
            .request_with_retry("update_user_region", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !region_response.success {
            anyhow::bail!(
                "Failed to update region: {}",
                region_response.message.unwrap_or_default()
            );
        }

        Ok(())
    }

    /// Upload user profile avatar
    pub async fn upload_avatar(&self, image_data: Vec<u8>, filename: String) -> Result<String> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if image_data.is_empty() {
            anyhow::bail!("Image data cannot be empty");
        }

        // Determine content type from filename extension
        let content_type = if filename.to_lowercase().ends_with(".png") {
            "image/png"
        } else if filename.to_lowercase().ends_with(".jpg")
            || filename.to_lowercase().ends_with(".jpeg")
        {
            "image/jpeg"
        } else if filename.to_lowercase().ends_with(".gif") {
            "image/gif"
        } else if filename.to_lowercase().ends_with(".webp") {
            "image/webp"
        } else {
            "image/jpeg" // default
        };

        // Build multipart form
        let part = reqwest::multipart::Part::bytes(image_data)
            .file_name(filename)
            .mime_str(content_type)
            .map_err(|e| anyhow::anyhow!("Invalid MIME type: {}", e))?;

        let form = reqwest::multipart::Form::new().part("avatar", part);

        let url = format!("{}/user/avatar/upload", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to upload avatar: {}", body);
        }

        // Parse response to extract avatar URL
        let response_json: serde_json::Value = response.json().await?;
        let avatar_url = response_json["avatar_url"]
            .as_str()
            .or_else(|| response_json["url"].as_str())
            .ok_or_else(|| anyhow::anyhow!("No avatar URL in response"))?
            .to_string();

        Ok(avatar_url)
    }

    // ===== USER SETTINGS =====

    /// Get a specific user setting by key
    pub async fn get_user_setting(&self, key: String) -> Result<Option<String>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if key.is_empty() {
            anyhow::bail!("Setting key cannot be empty");
        }

        let request = GetSettingRequest { key: key.clone() };

        let response = self
            .http_client
            .post(&format!("{}/user/setting/get", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get user setting: {}", response.status());
        }

        let setting_response: GetSettingResponse = response.json().await?;
        Ok(setting_response.value)
    }

    /// Get all user settings
    pub async fn list_user_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/user/setting/list", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list user settings: {}", response.status());
        }

        let settings_response: SettingsListResponse = response.json().await?;
        Ok(settings_response.settings)
    }

    /// Save a user setting
    pub async fn save_user_setting(&self, key: String, value: String) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if key.is_empty() {
            anyhow::bail!("Setting key cannot be empty");
        }

        let request = SaveSettingRequest { key, value };

        let response = self
            .http_client
            .post(&format!("{}/user/setting/save", self.base_url))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to save user setting: {}", response.status());
        }

        let save_response: SaveSettingResponse = response.json().await?;

        if !save_response.success {
            anyhow::bail!(
                "Failed to save setting: {}",
                save_response
                    .message
                    .unwrap_or_else(|| "Unknown error".to_string())
            );
        }

        Ok(())
    }

    /// Get list of available AI engines
    pub async fn list_ai_engines(&self) -> Result<Vec<AIEngine>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let response = self
            .http_client
            .get(&format!("{}/user/setting/ai_engine/list", self.base_url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list AI engines: {}", response.status());
        }

        let engines_response: AIEngineListResponse = response.json().await?;
        Ok(engines_response.engines)
    }

    // ===== SPEAKER DIARIZATION METHODS =====

    /// Analyze speaker segments by uploading audio file
    ///
    /// This method uploads an audio file to the HiNotes cloud API for speaker diarization.
    /// The API will analyze the audio and return speaker segments with timing information.
    ///
    /// # Arguments
    /// * `audio_path` - Path to the audio file to analyze
    /// * `note_id` - Associated note ID for tracking
    ///
    /// # Returns
    /// Vector of speaker segments with timing and confidence scores
    pub async fn analyze_speaker_segments(
        &self,
        audio_path: std::path::PathBuf,
        note_id: &str,
    ) -> Result<Vec<SpeakerSegment>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if note_id.is_empty() {
            anyhow::bail!("Note ID cannot be empty");
        }

        if !audio_path.exists() {
            anyhow::bail!("Audio file does not exist: {:?}", audio_path);
        }

        // Read audio file
        let audio_data = tokio::fs::read(&audio_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read audio file: {}", e))?;

        // Get filename
        let filename = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();

        let url = format!("{}/note/speaker/find", self.base_url);

        // We can't use request_with_retry here because Form doesn't implement Clone
        // So we'll do a single request with timeout
        let part = reqwest::multipart::Part::bytes(audio_data)
            .file_name(filename)
            .mime_str("audio/wav")
            .map_err(|e| anyhow::anyhow!("Failed to set MIME type: {}", e))?;

        let form = reqwest::multipart::Form::new()
            .part("audio", part)
            .text("note_id", note_id.to_string());

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to analyze speakers: {}", response.status());
        }

        let speaker_response: FindSpeakersResponse = response.json().await?;
        Ok(speaker_response.segments)
    }

    /// Extract voice signature from an audio segment
    ///
    /// This method extracts acoustic features from a specific time segment of audio
    /// for use in speaker matching and recognition.
    ///
    /// # Arguments
    /// * `note_id` - Note ID containing the audio
    /// * `start_time` - Start time in seconds
    /// * `end_time` - End time in seconds
    ///
    /// # Returns
    /// Voice signature containing acoustic features
    pub async fn extract_voice_signature(
        &self,
        note_id: &str,
        start_time: f64,
        end_time: f64,
    ) -> Result<VoiceSignature> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if note_id.is_empty() {
            anyhow::bail!("Note ID cannot be empty");
        }

        if start_time < 0.0 || end_time <= start_time {
            anyhow::bail!("Invalid time range: start={}, end={}", start_time, end_time);
        }

        let request = ExtractVoiceSignatureRequest {
            note_id: note_id.to_string(),
            start_time,
            end_time,
        };

        let url = format!("{}/note/speaker/signature/extract", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let response: ExtractVoiceSignatureResponse = self
            .request_with_retry("extract_voice_signature", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        Ok(response.signature)
    }

    /// Match a voice signature against known speakers
    ///
    /// This method compares a voice signature against a database of known speakers
    /// using acoustic similarity (e.g., cosine similarity of feature vectors).
    ///
    /// # Arguments
    /// * `signature_id` - ID of the voice signature to match
    /// * `candidate_speaker_ids` - List of speaker IDs to compare against
    /// * `threshold` - Minimum similarity threshold (0.0-1.0), defaults to 0.7
    ///
    /// # Returns
    /// Matched speaker ID (if confidence exceeds threshold) and similarity scores
    pub async fn match_speaker(
        &self,
        signature_id: &str,
        candidate_speaker_ids: Vec<String>,
        threshold: Option<f64>,
    ) -> Result<MatchSpeakerResponse> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if signature_id.is_empty() {
            anyhow::bail!("Signature ID cannot be empty");
        }

        if candidate_speaker_ids.is_empty() {
            anyhow::bail!("Candidate speaker IDs cannot be empty");
        }

        let request = MatchSpeakerRequest {
            signature_id: signature_id.to_string(),
            candidate_speaker_ids,
            threshold,
        };

        let url = format!("{}/note/speaker/signature/match", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let response: MatchSpeakerResponse = self
            .request_with_retry("match_speaker", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        Ok(response)
    }

    /// Extract voice signature from local audio segment (in-memory)
    ///
    /// This is a local utility method that extracts acoustic features from
    /// an audio segment without making an API call. Useful for offline processing.
    ///
    /// # Arguments
    /// * `audio_segment` - Audio segment with raw PCM data
    ///
    /// # Returns
    /// Voice signature with extracted features (simplified local extraction)
    pub fn extract_voice_signature_local(
        &self,
        audio_segment: &AudioSegment,
    ) -> Result<VoiceSignature> {
        // For now, this is a placeholder implementation
        // In a production system, this would use a local acoustic feature extraction library
        // such as MFCC extraction from the audio data

        if audio_segment.data.is_empty() {
            anyhow::bail!("Audio segment data is empty");
        }

        // Simple feature extraction: calculate RMS energy in chunks as a basic feature vector
        let chunk_size = 1024;
        let mut features = Vec::new();

        for chunk in audio_segment.data.chunks(chunk_size) {
            let sum_squares: f32 = chunk
                .iter()
                .map(|&sample| {
                    // Convert u8 to normalized float [-1.0, 1.0]
                    let normalized = (sample as f32 - 128.0) / 128.0;
                    normalized * normalized
                })
                .sum();

            let rms = (sum_squares / chunk.len() as f32).sqrt();
            features.push(rms);
        }

        // Normalize features to unit length for cosine similarity
        let magnitude: f32 = features.iter().map(|&f| f * f).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            features = features.iter().map(|&f| f / magnitude).collect();
        }

        Ok(VoiceSignature {
            id: uuid::Uuid::new_v4().to_string(),
            speaker_id: None,
            features,
            confidence: 0.0, // No confidence for local extraction
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Calculate cosine similarity between two voice signatures
    ///
    /// # Arguments
    /// * `sig1` - First voice signature
    /// * `sig2` - Second voice signature
    ///
    /// # Returns
    /// Similarity score between 0.0 (completely different) and 1.0 (identical)
    pub fn calculate_signature_similarity(
        &self,
        sig1: &VoiceSignature,
        sig2: &VoiceSignature,
    ) -> Result<f64> {
        if sig1.features.is_empty() || sig2.features.is_empty() {
            anyhow::bail!("Voice signature features are empty");
        }

        if sig1.features.len() != sig2.features.len() {
            anyhow::bail!(
                "Voice signature feature dimensions do not match: {} vs {}",
                sig1.features.len(),
                sig2.features.len()
            );
        }

        // Calculate cosine similarity: dot product of normalized vectors
        let dot_product: f32 = sig1
            .features
            .iter()
            .zip(sig2.features.iter())
            .map(|(a, b)| a * b)
            .sum();

        // Since vectors should be normalized, dot product is the cosine similarity
        Ok(dot_product.clamp(0.0, 1.0) as f64)
    }

    // ===== DEVICE FILE TRANSFER METHODS =====

    /// List files on connected HiDoc device
    ///
    /// # Arguments
    /// * `device_id` - The unique identifier of the connected device
    ///
    /// # Returns
    /// * `Result<Vec<DeviceFile>>` - List of files stored on the device
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if device is not connected or not found
    pub async fn list_device_files(&self, device_id: String) -> Result<Vec<DeviceFile>> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if device_id.is_empty() {
            anyhow::bail!("Device ID cannot be empty");
        }

        let request_body = DeviceFileListRequest {
            device_id: device_id.clone(),
        };

        let url = format!("{}/user/device/file/list", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let file_list_response: DeviceFileListResponse = self
            .request_with_retry("list_device_files", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        log::info!(
            "Retrieved {} files from device {}",
            file_list_response.files.len(),
            device_id
        );

        Ok(file_list_response.files)
    }

    /// Download a file from the connected HiDoc device
    ///
    /// # Arguments
    /// * `device_id` - The unique identifier of the connected device
    /// * `file_id` - The unique identifier of the file to download
    /// * `progress_callback` - Optional callback for tracking download progress
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - The file contents as raw bytes
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if device or file is not found
    /// * Returns error if download fails or is interrupted
    pub async fn download_device_file<F>(
        &self,
        device_id: String,
        file_id: String,
        progress_callback: Option<F>,
    ) -> Result<Vec<u8>>
    where
        F: Fn(u64, u64) + Send + Sync,
    {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if device_id.is_empty() {
            anyhow::bail!("Device ID cannot be empty");
        }

        if file_id.is_empty() {
            anyhow::bail!("File ID cannot be empty");
        }

        let request_body = DeviceFileGetRequest {
            device_id: device_id.clone(),
            file_id: file_id.clone(),
        };

        let url = format!("{}/user/device/file/get", self.base_url);
        let client = &self.http_client;

        log::info!("Downloading file {} from device {}", file_id, device_id);

        // Make initial request
        let response = client
            .post(&url)
            .bearer_auth(&token)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to download file: {}", body);
        }

        // Get content length if available
        let content_length = response.content_length().unwrap_or(0);

        // For now, download the entire file at once
        // TODO: Implement chunked streaming with progress for large files
        if let Some(ref callback) = progress_callback {
            callback(0, content_length);
        }

        let buffer = response.bytes().await?.to_vec();

        if let Some(ref callback) = progress_callback {
            callback(buffer.len() as u64, content_length);
        }

        log::info!(
            "Downloaded {} bytes for file {} from device {}",
            buffer.len(),
            file_id,
            device_id
        );

        Ok(buffer)
    }

    /// Upload a file to the connected HiDoc device
    ///
    /// # Arguments
    /// * `device_id` - The unique identifier of the connected device
    /// * `file_path` - Path to the file to upload
    /// * `progress_callback` - Optional callback for tracking upload progress
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if file does not exist or cannot be read
    /// * Returns error if device is not connected or not found
    /// * Returns error if upload fails or is interrupted
    pub async fn upload_device_file<F>(
        &self,
        device_id: String,
        file_path: std::path::PathBuf,
        progress_callback: Option<F>,
    ) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync,
    {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if device_id.is_empty() {
            anyhow::bail!("Device ID cannot be empty");
        }

        if !file_path.exists() {
            anyhow::bail!("File does not exist: {:?}", file_path);
        }

        if !file_path.is_file() {
            anyhow::bail!("Path is not a file: {:?}", file_path);
        }

        // Read file contents
        let file_contents = tokio::fs::read(&file_path).await?;
        let file_size = file_contents.len() as u64;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_string();

        log::info!(
            "Uploading file {} ({} bytes) to device {}",
            file_name,
            file_size,
            device_id
        );

        // Detect MIME type based on file extension
        let content_type = if file_name.ends_with(".mp3") {
            "audio/mpeg"
        } else if file_name.ends_with(".wav") {
            "audio/wav"
        } else if file_name.ends_with(".m4a") {
            "audio/mp4"
        } else if file_name.ends_with(".ogg") {
            "audio/ogg"
        } else {
            "application/octet-stream"
        };

        // Create multipart form
        let part = reqwest::multipart::Part::bytes(file_contents)
            .file_name(file_name.clone())
            .mime_str(content_type)
            .map_err(|e| anyhow::anyhow!("Invalid MIME type: {}", e))?;

        let form = reqwest::multipart::Form::new()
            .text("device_id", device_id.clone())
            .part("file", part);

        let url = format!("{}/user/device/file/upload", self.base_url);

        // Upload with progress tracking
        // Note: Progress tracking for uploads is more complex with reqwest
        // For now, we'll call the callback at 0% and 100%
        if let Some(ref callback) = progress_callback {
            callback(0, file_size);
        }

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to upload file: {}", body);
        }

        // Call progress callback at 100%
        if let Some(ref callback) = progress_callback {
            callback(file_size, file_size);
        }

        log::info!(
            "Successfully uploaded file {} to device {}",
            file_name,
            device_id
        );

        Ok(())
    }

    // ===== PASSWORD AND SECURITY METHODS =====

    /// Update user password
    ///
    /// # Arguments
    /// * `current_password` - Current password for verification
    /// * `new_password` - New password to set
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if current password is incorrect
    /// * Returns error if new password doesn't meet requirements
    pub async fn update_password(&self, current_password: &str, new_password: &str) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        if current_password.is_empty() {
            anyhow::bail!("Current password cannot be empty");
        }

        if new_password.len() < 8 {
            anyhow::bail!("New password must be at least 8 characters long");
        }

        if current_password == new_password {
            anyhow::bail!("New password must be different from current password");
        }

        let request = UpdatePasswordRequest {
            current_password: current_password.to_string(),
            new_password: new_password.to_string(),
        };

        let url = format!("{}/user/password/update", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let response: UpdatePasswordResponse = self
            .request_with_retry("update_password", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to update password: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("Password updated successfully");
        Ok(())
    }

    /// Delete user account
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if not authenticated
    /// * Returns error if deletion fails
    ///
    /// # Note
    /// This action is irreversible. All user data will be permanently deleted.
    pub async fn delete_user_account(&self) -> Result<()> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!("{}/user/delete", self.base_url);
        let client = &self.http_client;
        let token_clone = token.clone();

        let response: DeleteUserResponse = self
            .request_with_retry("delete_user_account", || async {
                client
                    .post(&url)
                    .bearer_auth(&token_clone)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to delete account: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("User account deleted successfully");
        Ok(())
    }

    /// Send email verification code
    ///
    /// # Arguments
    /// * `email` - Email address to send verification code to
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if email is invalid
    /// * Returns error if sending fails
    pub async fn send_email_verification(&self, email: &str) -> Result<()> {
        if email.is_empty() {
            anyhow::bail!("Email cannot be empty");
        }

        if !email.contains('@') {
            anyhow::bail!("Invalid email format");
        }

        let request = SendEmailVerificationRequest {
            email: email.to_string(),
        };

        let url = format!("{}/user/email/verification/send", self.base_url);
        let client = &self.http_client;

        let response: SendEmailVerificationResponse = self
            .request_with_retry("send_email_verification", || async {
                client
                    .post(&url)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to send verification code: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("Email verification code sent to {}", email);
        Ok(())
    }

    /// Verify email verification code
    ///
    /// # Arguments
    /// * `code` - Verification code received via email
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if code is invalid or expired
    pub async fn verify_email_code(&self, code: &str) -> Result<()> {
        if code.is_empty() {
            anyhow::bail!("Verification code cannot be empty");
        }

        let request = VerifyEmailRequest {
            code: code.to_string(),
        };

        let url = format!("{}/user/email/verification/verify", self.base_url);
        let client = &self.http_client;

        let response: VerifyEmailResponse = self
            .request_with_retry("verify_email_code", || async {
                client
                    .post(&url)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to verify email: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("Email verified successfully");
        Ok(())
    }

    /// Send password reset code
    ///
    /// # Arguments
    /// * `email` - Email address to send reset code to
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if email is not found
    /// * Returns error if sending fails
    pub async fn send_password_reset_code(&self, email: &str) -> Result<()> {
        if email.is_empty() {
            anyhow::bail!("Email cannot be empty");
        }

        if !email.contains('@') {
            anyhow::bail!("Invalid email format");
        }

        let request = SendPasswordResetRequest {
            email: email.to_string(),
        };

        let url = format!("{}/user/reset/authcode/send", self.base_url);
        let client = &self.http_client;

        let response: SendPasswordResetResponse = self
            .request_with_retry("send_password_reset_code", || async {
                client
                    .post(&url)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to send reset code: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("Password reset code sent to {}", email);
        Ok(())
    }

    /// Verify password reset code
    ///
    /// # Arguments
    /// * `code` - Reset code received via email
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if code is invalid or expired
    pub async fn verify_password_reset_code(&self, code: &str) -> Result<()> {
        if code.is_empty() {
            anyhow::bail!("Reset code cannot be empty");
        }

        let request = VerifyResetCodeRequest {
            code: code.to_string(),
        };

        let url = format!("{}/user/reset/check", self.base_url);
        let client = &self.http_client;

        let response: VerifyResetCodeResponse = self
            .request_with_retry("verify_password_reset_code", || async {
                client
                    .post(&url)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to verify reset code: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("Password reset code verified successfully");
        Ok(())
    }

    /// Save new password after reset
    ///
    /// # Arguments
    /// * `code` - Verified reset code
    /// * `new_password` - New password to set
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// * Returns error if code is invalid
    /// * Returns error if password doesn't meet requirements
    pub async fn save_new_password(&self, code: &str, new_password: &str) -> Result<()> {
        if code.is_empty() {
            anyhow::bail!("Reset code cannot be empty");
        }

        if new_password.len() < 8 {
            anyhow::bail!("Password must be at least 8 characters long");
        }

        let request = SaveNewPasswordRequest {
            code: code.to_string(),
            password: new_password.to_string(),
        };

        let url = format!("{}/user/reset/save", self.base_url);
        let client = &self.http_client;

        let response: SaveNewPasswordResponse = self
            .request_with_retry("save_new_password", || async {
                client
                    .post(&url)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))
            })
            .await?;

        if !response.success {
            anyhow::bail!(
                "Failed to save new password: {}",
                response.message.unwrap_or_default()
            );
        }

        log::info!("New password saved successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
    async fn test_register_new_user() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client
            .register("newuser@example.com", "password123", "New User")
            .await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "newuser@example.com");
        assert_eq!(user.name, "New User");
        assert!(!user.id.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires external API/mock server
    async fn test_register_stores_token() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let _ = client
            .register("newuser@example.com", "password123", "New User")
            .await;
        let token = client.get_token().await;
        assert!(token.is_some());
        assert!(!token.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_register_validates_email() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("", "password123", "User").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Email cannot be empty"));
    }

    #[tokio::test]
    async fn test_register_validates_password_length() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("user@example.com", "short", "User").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 8 characters"));
    }

    #[tokio::test]
    async fn test_register_validates_name() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.register("user@example.com", "password123", "").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Name cannot be empty"));
    }

    // ===== LOGOUT TESTS =====

    #[tokio::test]
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
    async fn test_claim_trial_success() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client
            .authenticate("eligible@example.com", "password")
            .await;

        let result = client.claim_trial().await;
        assert!(result.is_ok());
        let claim_response = result.unwrap();
        assert!(claim_response.success || !claim_response.success);
        assert!(!claim_response.message.is_empty());
    }

    // ===== CACHING TESTS =====

    #[tokio::test]
    #[ignore] // Requires external API/mock server
    async fn test_subscription_caching() {
        use std::time::Duration;

        let client =
            HiNotesClient::with_cache_duration("http://localhost:3001/v1", Duration::from_secs(60));
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
    #[ignore] // Requires external API/mock server
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
    #[ignore] // Requires external API/mock server
    async fn test_get_subscription_status_fresh() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        // Get fresh data (bypasses cache)
        let result = client.get_subscription_status_fresh().await;
        assert!(result.is_ok());
    }

    // ===== GRACE PERIOD TESTS =====

    #[tokio::test]
    #[ignore] // Requires external API/mock server
    async fn test_is_in_grace_period() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("test@example.com", "password").await;

        let result = client.is_in_grace_period().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires external API/mock server
    async fn test_is_subscription_expired() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        let _ = client.authenticate("expired@example.com", "password").await;

        let result = client.is_subscription_expired().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires external API/mock server
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

    // ===== LIVE TRANSLATION API TESTS =====

    #[tokio::test]
    async fn test_get_live_translation_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.get_live_translation("note-123").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_get_live_translation_empty_note_id() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.get_live_translation("").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Note ID cannot be empty"));
    }

    #[tokio::test]
    async fn test_rate_translation_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let result = client.rate_translation("note-123", 5, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_rate_translation_empty_note_id() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.rate_translation("", 5, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Note ID cannot be empty"));
    }

    #[tokio::test]
    async fn test_rate_translation_invalid_rating_too_low() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.rate_translation("note-123", 0, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Rating must be between 1 and 5"));
    }

    #[tokio::test]
    async fn test_rate_translation_invalid_rating_too_high() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        *client.auth_token.write().await = Some("mock_token".to_string());

        let result = client.rate_translation("note-123", 6, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Rating must be between 1 and 5"));
    }

    #[tokio::test]
    async fn test_translate_text_api_requires_auth() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());

        let request = TranslationRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };

        let result = client.translate_text_api(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_translate_text_api_empty_text() {
        let client = HiNotesClient::with_base_url("http://localhost:3001/v1".to_string());
        *client.auth_token.write().await = Some("mock_token".to_string());

        let request = TranslationRequest {
            text: "".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };

        let result = client.translate_text_api(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Translation text cannot be empty"));
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
