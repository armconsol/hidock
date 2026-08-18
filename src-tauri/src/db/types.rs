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
    pub synced_at: Option<DateTime<Utc>>,
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
