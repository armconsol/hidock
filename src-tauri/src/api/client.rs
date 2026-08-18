use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{
    AuthResponse, CalendarEventsResponse, CreateEventRequest, GoogleCalendarEvent, LoginRequest,
    UserInfo,
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
}
