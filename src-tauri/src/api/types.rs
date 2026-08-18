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

// Subscription & RevenueCat Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub product_id: String,
    pub status: SubscriptionStatus,
    pub expires_at: Option<String>,
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
