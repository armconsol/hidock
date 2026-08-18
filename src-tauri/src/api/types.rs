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
