use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub folder_id: Option<String>,
    pub audio_url: Option<String>,
    pub duration: Option<String>,
    pub rating: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertNote {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub folder_id: Option<String>,
    pub audio_url: Option<String>,
    pub duration: Option<String>,
    pub rating: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub content: Option<String>,
    pub folder_id: Option<String>,
    pub audio_url: Option<String>,
    pub duration: Option<String>,
    pub rating: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhisperNote {
    pub id: String,
    pub content: String,
    pub audio_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertWhisperNote {
    pub id: String,
    pub content: String,
    pub audio_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: String,
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub state: TodoState,
    pub smart_label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTodo {
    pub id: String,
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub state: TodoState,
    pub smart_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub state: Option<TodoState>,
    pub smart_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TodoState {
    Open,
    Closed,
}

impl TodoState {
    pub fn as_str(&self) -> &str {
        match self {
            TodoState::Open => "open",
            TodoState::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "open" => Ok(TodoState::Open),
            "closed" => Ok(TodoState::Closed),
            _ => Err(format!("Invalid todo state: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub source: EventSource,
    pub meeting_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub google_event_id: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    GoogleCalendar,
    Hinotes,
}

impl EventSource {
    pub fn as_str(&self) -> &str {
        match self {
            EventSource::GoogleCalendar => "google_calendar",
            EventSource::Hinotes => "hinotes",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "google_calendar" => Ok(EventSource::GoogleCalendar),
            "hinotes" => Ok(EventSource::Hinotes),
            _ => Err(format!("Invalid event source: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Synced,
    PendingPush,
    Conflict,
}

impl SyncStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SyncStatus::Synced => "synced",
            SyncStatus::PendingPush => "pending_push",
            SyncStatus::Conflict => "conflict",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "synced" => Ok(SyncStatus::Synced),
            "pending_push" => Ok(SyncStatus::PendingPush),
            "conflict" => Ok(SyncStatus::Conflict),
            _ => Err(format!("Invalid sync status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Template {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_favorite: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

// Pagination parameters
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub limit: i64,
    pub offset: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

impl PaginationParams {
    pub fn new(limit: i64, offset: i64) -> Self {
        Self { limit, offset }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartLabel {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSmartLabel {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSmartLabel {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vocabulary {
    pub id: String,
    pub word: String,
    pub pronunciation: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertVocabulary {
    pub id: String,
    pub word: String,
    pub pronunciation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub status: DeviceStatus,
    pub last_sync: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Connected,
    Disconnected,
}

impl DeviceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            DeviceStatus::Connected => "connected",
            DeviceStatus::Disconnected => "disconnected",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "connected" => Ok(DeviceStatus::Connected),
            "disconnected" => Ok(DeviceStatus::Disconnected),
            _ => Err(format!("Invalid device status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShareLink {
    pub id: String,
    pub note_id: String,
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub access_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertShareLink {
    pub id: String,
    pub note_id: String,
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
}

// Speaker Diarization Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Speaker {
    pub id: String,
    pub name: Option<String>,
    pub voice_signature: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSpeaker {
    pub id: String,
    pub name: Option<String>,
    pub voice_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSpeaker {
    pub name: Option<String>,
    pub voice_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerSegment {
    pub id: String,
    pub note_id: String,
    pub speaker_id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSpeakerSegment {
    pub id: String,
    pub note_id: String,
    pub speaker_id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
}

// Subscription Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbSubscription {
    pub id: i64,
    pub product_id: String,
    pub status: SubscriptionStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub purchased_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSubscription {
    pub product_id: String,
    pub status: SubscriptionStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub purchased_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscription {
    pub status: Option<SubscriptionStatus>,
    pub expires_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
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
pub struct SubscriptionEvent {
    pub id: i64,
    pub subscription_id: Option<i64>,
    pub event_type: SubscriptionEventType,
    pub product_id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSubscriptionEvent {
    pub subscription_id: Option<i64>,
    pub event_type: SubscriptionEventType,
    pub product_id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionEventType {
    Activated,
    Expired,
    Renewed,
    Canceled,
}

impl SubscriptionEventType {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionEventType::Activated => "activated",
            SubscriptionEventType::Expired => "expired",
            SubscriptionEventType::Renewed => "renewed",
            SubscriptionEventType::Canceled => "canceled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "activated" => Ok(SubscriptionEventType::Activated),
            "expired" => Ok(SubscriptionEventType::Expired),
            "renewed" => Ok(SubscriptionEventType::Renewed),
            "canceled" => Ok(SubscriptionEventType::Canceled),
            _ => Err(format!("Invalid subscription event type: {}", s)),
        }
    }
}
