use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub token: String,
    pub user: UserInfo,
    pub message: Option<String>,
}

// Password and Security Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    #[serde(rename = "currentPassword")]
    pub current_password: String,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailVerificationRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailVerificationResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmailRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmailResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPasswordResetResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResetCodeRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResetCodeResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveNewPasswordRequest {
    pub code: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveNewPasswordResponse {
    pub success: bool,
    pub message: Option<String>,
}

// Google Calendar Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleCalendarEvent {
    pub id: String,
    pub summary: String,
    pub start: EventDateTime,
    pub end: EventDateTime,
    #[serde(rename = "htmlLink")]
    pub html_link: Option<String>,
    #[serde(rename = "hangoutLink")]
    pub hangout_link: Option<String>,
    pub updated: Option<String>, // RFC3339 timestamp of last modification
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventsResponse {
    pub items: Vec<GoogleCalendarEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub summary: String,
    pub start: EventDateTime,
    pub end: EventDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    pub summary: String,
    pub start: EventDateTime,
    pub end: EventDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRecordingStatusRequest {
    #[serde(rename = "eventId")]
    pub event_id: String,
    #[serde(rename = "isRecording")]
    pub is_recording: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRecordingStatusResponse {
    pub success: bool,
    pub message: Option<String>,
}

// Translation Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationRequest {
    pub text: String,
    #[serde(rename = "sourceLang")]
    pub source_lang: String,
    #[serde(rename = "targetLang")]
    pub target_lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationResponse {
    #[serde(rename = "translatedText")]
    pub translated_text: String,
    #[serde(rename = "sourceLang")]
    pub source_lang: String,
    #[serde(rename = "targetLang")]
    pub target_lang: String,
    #[serde(rename = "detectedLang")]
    pub detected_lang: Option<String>,
    pub confidence: Option<f64>,
}

impl TranslationResponse {
    /// Create from cached translation
    pub fn from_cache(
        _source_text: &str,
        source_lang: &str,
        target_lang: &str,
        translated_text: &str,
    ) -> Self {
        Self {
            translated_text: translated_text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            detected_lang: None,
            confidence: Some(1.0), // Cached translations are assumed correct
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Language {
    pub code: String,
    pub name: String,
    #[serde(rename = "nativeName")]
    pub native_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageListResponse {
    pub languages: Vec<Language>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectLanguageRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectLanguageResponse {
    #[serde(rename = "detectedLang")]
    pub detected_lang: String,
    pub confidence: f64,
}

// Speaker Diarization Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerSegment {
    pub id: String,
    pub note_id: String,
    pub speaker_id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerProfile {
    pub id: String,
    pub name: Option<String>,
    pub voice_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindSpeakersRequest {
    pub note_id: String,
    pub audio_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindSpeakersResponse {
    pub note_id: String,
    pub speakers: Vec<SpeakerProfile>,
    pub segments: Vec<SpeakerSegment>,
}

/// Audio segment for voice signature extraction
#[derive(Debug, Clone)]
pub struct AudioSegment {
    pub data: Vec<u8>,
    pub start_time: f64,
    pub end_time: f64,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Voice signature for speaker matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceSignature {
    /// Unique identifier for the voice signature
    pub id: String,
    /// Associated speaker ID (if matched)
    pub speaker_id: Option<String>,
    /// Acoustic feature vector (e.g., MFCCs, i-vectors, x-vectors)
    pub features: Vec<f32>,
    /// Confidence score for this signature (0.0-1.0)
    pub confidence: f64,
    /// Timestamp when signature was created
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractVoiceSignatureRequest {
    pub note_id: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractVoiceSignatureResponse {
    pub signature: VoiceSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchSpeakerRequest {
    pub signature_id: String,
    pub candidate_speaker_ids: Vec<String>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchSpeakerResponse {
    pub matched_speaker_id: Option<String>,
    pub confidence: f64,
    pub similarity_scores: Vec<SpeakerSimilarity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerSimilarity {
    pub speaker_id: String,
    pub similarity_score: f64,
}

// Subscription & RevenueCat Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub product_id: String,
    pub status: SubscriptionStatus,
    pub expires_at: Option<String>,
    /// True if subscription is in grace period (expired but still accessible)
    #[serde(default)]
    pub in_grace_period: bool,
    /// Number of days remaining in grace period (if applicable)
    #[serde(default)]
    pub grace_period_days_remaining: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Canceled,
    Trial,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Expired => "expired",
            SubscriptionStatus::Canceled => "canceled",
            SubscriptionStatus::Trial => "trial",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(SubscriptionStatus::Active),
            "expired" => Ok(SubscriptionStatus::Expired),
            "canceled" => Ok(SubscriptionStatus::Canceled),
            "trial" => Ok(SubscriptionStatus::Trial),
            _ => Err(format!("Invalid subscription status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferralInfo {
    pub code: String,
    pub rewards_earned: f64,
    pub referrals_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub subscriber: SubscriberInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberInfo {
    pub entitlements: Entitlements,
    pub subscriptions: std::collections::HashMap<String, SubscriptionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlements {
    pub premium: Option<EntitlementInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementInfo {
    pub expires_date: Option<String>,
    pub product_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDetail {
    pub expires_date: Option<String>,
    pub purchase_date: String,
    pub original_purchase_date: String,
    pub store: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralOverviewResponse {
    pub referral_code: String,
    pub total_rewards: f64,
    pub total_referrals: i32,
    pub pending_rewards: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub event_type: SubscriptionEventType,
    pub product_id: String,
    pub expires_at: Option<String>,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubscriptionEventType {
    Activated,
    Expired,
    Renewed,
    Canceled,
}

// Reward Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailableReward {
    pub id: String,
    pub reward_type: String,
    pub amount: f64,
    pub description: String,
    pub points_required: f64,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsListResponse {
    pub rewards: Vec<AvailableReward>,
    pub total_points: f64,
    pub available_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemRewardRequest {
    pub reward_id: String,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemRewardResponse {
    pub success: bool,
    pub reward_id: String,
    pub points_used: f64,
    pub remaining_points: f64,
    pub message: String,
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
    pub paypal_email: String,
    pub estimated_completion: Option<String>,
}

// Receipt Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub id: String,
    pub product_id: String,
    pub purchase_date: String,
    pub store: String,
    pub amount: f64,
    pub currency: String,
    pub is_trial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptsResponse {
    pub receipts: Vec<Receipt>,
}

// Billing Portal Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPortalResponse {
    pub url: String,
}

// Trial Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrialEligibility {
    pub eligible: bool,
    pub reason: Option<String>,
    pub trial_duration_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimTrialResponse {
    pub success: bool,
    pub subscription: Option<Subscription>,
    pub expires_at: Option<String>,
    pub message: String,
}

// Device File Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceFile {
    pub file_id: String,
    pub name: String,
    pub size: i64,
    pub date: String,
    pub duration: Option<f64>,
    pub already_synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFileListRequest {
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFileListResponse {
    pub files: Vec<DeviceFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFileGetRequest {
    pub device_id: String,
    pub file_id: String,
}

// Settings Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSettingRequest {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSettingResponse {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsListResponse {
    pub settings: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSettingRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSettingResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AIEngine {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub requires_subscription: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEngineListResponse {
    pub engines: Vec<AIEngine>,
}

// User Profile Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameUserRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameUserResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRegionRequest {
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRegionResponse {
    pub success: bool,
    pub message: Option<String>,
}
