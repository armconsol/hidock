use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};

use crate::api::client::HiNotesClient;
use crate::api::types::{
    CreateEventRequest, EventDateTime, GoogleCalendarEvent, UpdateEventRequest,
};
use crate::db::types::{CalendarEvent, EventSource, SyncStatus};
use crate::db::Database;

const DEFAULT_SYNC_INTERVAL_SECS: u64 = 30;
const CONNECTIVITY_CHECK_URL: &str = "https://www.googleapis.com/calendar/v3/users/me/settings";
const MAX_RETRIES: u32 = 3;
const SYNC_WINDOW_DAYS_PAST: i64 = 7;
const SYNC_WINDOW_DAYS_FUTURE: i64 = 30;

/// Background calendar sync worker
///
/// Periodically fetches calendar events from Google Calendar API
/// and synchronizes them with the local database.
pub struct CalendarSync {
    /// Sync interval duration (default: 30 seconds)
    sync_interval: std::time::Duration,
    /// Last sync timestamp
    last_sync: Arc<Mutex<Option<DateTime<Utc>>>>,
    /// Google OAuth token (if authenticated with Google)
    google_token: Arc<Mutex<Option<String>>>,
    /// HiNotes API client
    api_client: Arc<HiNotesClient>,
    /// Local database
    db: Arc<Mutex<Database>>,
    /// Worker running state
    is_running: Arc<Mutex<bool>>,
    /// Calendar ID to sync (default: "primary")
    calendar_id: Arc<Mutex<String>>,
}

impl CalendarSync {
    /// Create a new calendar sync worker
    pub fn new(api_client: Arc<HiNotesClient>, db: Arc<Mutex<Database>>) -> Self {
        Self {
            sync_interval: std::time::Duration::from_secs(DEFAULT_SYNC_INTERVAL_SECS),
            last_sync: Arc::new(Mutex::new(None)),
            google_token: Arc::new(Mutex::new(None)),
            api_client,
            db,
            is_running: Arc::new(Mutex::new(false)),
            calendar_id: Arc::new(Mutex::new("primary".to_string())),
        }
    }

    /// Create a new calendar sync worker with custom sync interval
    pub fn with_interval(
        api_client: Arc<HiNotesClient>,
        db: Arc<Mutex<Database>>,
        interval_secs: u64,
    ) -> Self {
        let mut sync = Self::new(api_client, db);
        sync.sync_interval = std::time::Duration::from_secs(interval_secs);
        sync
    }

    /// Set the Google OAuth token for calendar access
    pub async fn set_google_token(&self, token: String) {
        let mut google_token = self.google_token.lock().await;
        *google_token = Some(token);
        log::info!("Google Calendar token set");
    }

    /// Clear the Google OAuth token
    pub async fn clear_google_token(&self) {
        let mut google_token = self.google_token.lock().await;
        *google_token = None;
        log::info!("Google Calendar token cleared");
    }

    /// Set the calendar ID to sync (default: "primary")
    pub async fn set_calendar_id(&self, calendar_id: String) {
        let mut id = self.calendar_id.lock().await;
        *id = calendar_id;
        log::info!("Calendar ID set to: {}", id);
    }

    /// Start the background sync loop
    pub async fn start_sync_loop(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            anyhow::bail!("Calendar sync worker is already running");
        }
        *is_running = true;
        drop(is_running);

        log::info!("Starting calendar sync worker");

        let sync_interval = self.sync_interval;
        let last_sync = self.last_sync.clone();
        let google_token = self.google_token.clone();
        let api_client = self.api_client.clone();
        let db = self.db.clone();
        let is_running = self.is_running.clone();
        let calendar_id = self.calendar_id.clone();

        tokio::spawn(async move {
            let mut tick = interval(sync_interval);

            loop {
                tick.tick().await;

                // Check if worker should still be running
                {
                    let running = is_running.lock().await;
                    if !*running {
                        log::info!("Calendar sync worker stopping");
                        break;
                    }
                }

                // Check if Google token is available
                let token = {
                    let token_guard = google_token.lock().await;
                    token_guard.clone()
                };

                if token.is_none() {
                    log::debug!("No Google token available, skipping calendar sync");
                    continue;
                }

                // Check connectivity to Google Calendar API
                if !Self::check_connectivity(&api_client).await {
                    log::debug!("Cannot reach Google Calendar API, skipping sync");
                    continue;
                }

                // Perform sync
                let cal_id = {
                    let id_guard = calendar_id.lock().await;
                    id_guard.clone()
                };

                match Self::sync_events_internal(&api_client, &db, &cal_id, &last_sync).await {
                    Ok(count) => {
                        if count > 0 {
                            log::info!("Synced {} calendar event(s)", count);
                        }
                    }
                    Err(e) => {
                        log::error!("Calendar sync error: {}", e);
                    }
                }
            }

            log::info!("Calendar sync worker stopped");
        });

        Ok(())
    }

    /// Stop the sync worker
    pub async fn stop(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        log::info!("Calendar sync worker stop requested");
    }

    /// Check if the worker is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// Get the last sync timestamp
    pub async fn get_last_sync(&self) -> Option<DateTime<Utc>> {
        *self.last_sync.lock().await
    }

    /// Manually trigger a sync
    pub async fn sync_now(&self) -> Result<usize> {
        let token = self.google_token.lock().await;
        if token.is_none() {
            anyhow::bail!("No Google token available");
        }
        drop(token);

        let cal_id = self.calendar_id.lock().await.clone();

        Self::sync_events_internal(&self.api_client, &self.db, &cal_id, &self.last_sync).await
    }

    /// Internal sync implementation - bidirectional sync
    async fn sync_events_internal(
        api_client: &Arc<HiNotesClient>,
        db: &Arc<Mutex<Database>>,
        calendar_id: &str,
        last_sync: &Arc<Mutex<Option<DateTime<Utc>>>>,
    ) -> Result<usize> {
        let now = Utc::now();

        // Define sync window: past 7 days to future 30 days
        let time_min = now - Duration::days(SYNC_WINDOW_DAYS_PAST);
        let time_max = now + Duration::days(SYNC_WINDOW_DAYS_FUTURE);

        log::debug!(
            "Syncing calendar events from {} to {}",
            time_min.to_rfc3339(),
            time_max.to_rfc3339()
        );

        // Step 1: Push local changes to Google Calendar
        let pushed_count = Self::push_local_changes(api_client, db, calendar_id).await?;
        if pushed_count > 0 {
            log::info!("Pushed {} local changes to Google Calendar", pushed_count);
        }

        // Step 2: Fetch events from Google Calendar with retry logic
        let remote_events =
            Self::fetch_with_retry(api_client, calendar_id, time_min, time_max).await?;

        log::debug!(
            "Fetched {} events from Google Calendar",
            remote_events.len()
        );

        // Step 3: Pull remote changes and handle conflicts
        let pulled_count = Self::pull_remote_changes(db, &remote_events).await?;
        if pulled_count > 0 {
            log::info!("Pulled {} changes from Google Calendar", pulled_count);
        }

        // Update last sync timestamp
        {
            let mut last_sync_guard = last_sync.lock().await;
            *last_sync_guard = Some(now);
        }

        Ok(pushed_count + pulled_count)
    }

    /// Push local changes to Google Calendar
    async fn push_local_changes(
        api_client: &Arc<HiNotesClient>,
        db: &Arc<Mutex<Database>>,
        calendar_id: &str,
    ) -> Result<usize> {
        let db_lock = db.lock().await;
        let pending_events = db_lock.get_pending_push_events()?;
        drop(db_lock);

        let mut pushed_count = 0;

        for local_event in pending_events {
            match Self::push_event_to_google(api_client, db, calendar_id, &local_event).await {
                Ok(_) => {
                    pushed_count += 1;
                }
                Err(e) => {
                    log::error!(
                        "Failed to push event '{}' to Google: {}",
                        local_event.title,
                        e
                    );
                }
            }
        }

        Ok(pushed_count)
    }

    /// Push a single event to Google Calendar
    async fn push_event_to_google(
        api_client: &Arc<HiNotesClient>,
        db: &Arc<Mutex<Database>>,
        calendar_id: &str,
        local_event: &CalendarEvent,
    ) -> Result<()> {
        if local_event.google_event_id.is_none() {
            // Create new event
            let create_request = CreateEventRequest {
                summary: local_event.title.clone(),
                start: Self::datetime_to_event_datetime(local_event.start_time),
                end: Self::datetime_to_event_datetime(local_event.end_time),
            };

            let created_event = api_client.add_event(calendar_id, create_request).await?;

            // Update local event with Google ID and mark as synced
            let db_lock = db.lock().await;
            let mut updated_event = local_event.clone();
            updated_event.google_event_id = Some(created_event.id);
            updated_event.sync_status = SyncStatus::Synced;
            updated_event.synced_at = Some(Utc::now());
            db_lock.update_calendar_event(&updated_event)?;
            drop(db_lock);

            log::debug!("Created event '{}' in Google Calendar", local_event.title);
        } else {
            // Update existing event
            let google_id = local_event.google_event_id.as_ref().unwrap();
            let update_request = UpdateEventRequest {
                summary: local_event.title.clone(),
                start: Self::datetime_to_event_datetime(local_event.start_time),
                end: Self::datetime_to_event_datetime(local_event.end_time),
            };

            api_client
                .update_event(calendar_id, google_id, update_request)
                .await?;

            // Mark as synced
            let db_lock = db.lock().await;
            db_lock.update_calendar_event_sync_status(&local_event.id, SyncStatus::Synced)?;
            db_lock.mark_calendar_event_synced(&local_event.id)?;
            drop(db_lock);

            log::debug!("Updated event '{}' in Google Calendar", local_event.title);
        }

        Ok(())
    }

    /// Pull remote changes from Google Calendar
    async fn pull_remote_changes(
        db: &Arc<Mutex<Database>>,
        remote_events: &[GoogleCalendarEvent],
    ) -> Result<usize> {
        let db_lock = db.lock().await;
        let mut changed_count = 0;

        for remote_event in remote_events {
            match Self::sync_remote_event(&db_lock, remote_event) {
                Ok(changed) => {
                    if changed {
                        changed_count += 1;
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to sync event '{}' ({}): {}",
                        remote_event.summary,
                        remote_event.id,
                        e
                    );
                }
            }
        }

        Ok(changed_count)
    }

    /// Sync a single remote event with conflict resolution
    fn sync_remote_event(db: &Database, remote_event: &GoogleCalendarEvent) -> Result<bool> {
        // Parse timestamps
        let start_time = Self::parse_event_datetime(&remote_event.start)?;
        let end_time = Self::parse_event_datetime(&remote_event.end)?;

        let remote_updated = if let Some(ref updated_str) = remote_event.updated {
            DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        } else {
            None
        };

        // Determine meeting URL
        let meeting_url = remote_event
            .hangout_link
            .clone()
            .or_else(|| remote_event.html_link.clone());

        // Check if event already exists locally
        let existing = db.get_calendar_event_by_google_id(&remote_event.id)?;

        let now = Utc::now();

        if let Some(existing_event) = existing {
            // Event exists - check for conflicts
            if existing_event.sync_status == SyncStatus::PendingPush {
                // Local changes pending - conflict resolution
                return Self::resolve_conflict(db, &existing_event, remote_event, remote_updated);
            }

            // Check if update is needed
            if existing_event.title != remote_event.summary
                || existing_event.start_time != start_time
                || existing_event.end_time != end_time
                || existing_event.meeting_url != meeting_url
            {
                let mut updated_event = existing_event;
                updated_event.title = remote_event.summary.clone();
                updated_event.start_time = start_time;
                updated_event.end_time = end_time;
                updated_event.meeting_url = meeting_url;
                updated_event.updated_at = now;
                updated_event.synced_at = Some(now);
                updated_event.sync_status = SyncStatus::Synced;

                db.update_calendar_event(&updated_event)?;
                log::debug!(
                    "Updated calendar event from Google: {}",
                    updated_event.title
                );
                Ok(true)
            } else {
                // Event unchanged, just update synced_at
                db.mark_calendar_event_synced(&existing_event.id)?;
                Ok(false)
            }
        } else {
            // New event from Google - insert it
            let event = CalendarEvent {
                id: uuid::Uuid::new_v4().to_string(),
                title: remote_event.summary.clone(),
                start_time,
                end_time,
                source: EventSource::GoogleCalendar,
                meeting_url,
                created_at: now,
                updated_at: now,
                synced_at: Some(now),
                google_event_id: Some(remote_event.id.clone()),
                sync_status: SyncStatus::Synced,
            };

            db.insert_calendar_event(&event)?;
            log::debug!("Inserted new calendar event from Google: {}", event.title);
            Ok(true)
        }
    }

    /// Resolve conflict between local and remote changes (last-write-wins)
    fn resolve_conflict(
        db: &Database,
        local_event: &CalendarEvent,
        remote_event: &GoogleCalendarEvent,
        remote_updated: Option<DateTime<Utc>>,
    ) -> Result<bool> {
        // Compare timestamps - last write wins
        let should_keep_local = if let Some(remote_ts) = remote_updated {
            local_event.updated_at > remote_ts
        } else {
            // If no remote timestamp, prefer local
            true
        };

        if should_keep_local {
            // Keep local version, mark as pending push
            log::warn!(
                "Conflict for event '{}': keeping local version (local newer)",
                local_event.title
            );
            // The event will be pushed in the next sync cycle
            Ok(false)
        } else {
            // Accept remote version
            log::warn!(
                "Conflict for event '{}': accepting remote version (remote newer)",
                local_event.title
            );

            let start_time = Self::parse_event_datetime(&remote_event.start)?;
            let end_time = Self::parse_event_datetime(&remote_event.end)?;
            let meeting_url = remote_event
                .hangout_link
                .clone()
                .or_else(|| remote_event.html_link.clone());

            let mut updated_event = local_event.clone();
            updated_event.title = remote_event.summary.clone();
            updated_event.start_time = start_time;
            updated_event.end_time = end_time;
            updated_event.meeting_url = meeting_url;
            updated_event.updated_at = Utc::now();
            updated_event.synced_at = Some(Utc::now());
            updated_event.sync_status = SyncStatus::Synced;

            db.update_calendar_event(&updated_event)?;
            Ok(true)
        }
    }

    /// Convert DateTime to EventDateTime
    fn datetime_to_event_datetime(dt: DateTime<Utc>) -> EventDateTime {
        EventDateTime {
            date_time: Some(dt.to_rfc3339()),
            date: None,
            time_zone: None,
        }
    }

    /// Fetch events with retry logic
    async fn fetch_with_retry(
        api_client: &Arc<HiNotesClient>,
        calendar_id: &str,
        time_min: DateTime<Utc>,
        time_max: DateTime<Utc>,
    ) -> Result<Vec<GoogleCalendarEvent>> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            match api_client
                .list_events(calendar_id, time_min, time_max)
                .await
            {
                Ok(events) => return Ok(events),
                Err(e) => {
                    if attempts >= MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Failed to fetch calendar events after {} attempts: {}",
                            MAX_RETRIES,
                            e
                        ));
                    }

                    let backoff_duration = std::time::Duration::from_secs(2u64.pow(attempts - 1));
                    log::warn!(
                        "Failed to fetch events (attempt {}/{}): {}. Retrying in {:?}",
                        attempts,
                        MAX_RETRIES,
                        e,
                        backoff_duration
                    );
                    sleep(backoff_duration).await;
                }
            }
        }
    }

    /// Parse EventDateTime to DateTime<Utc>
    fn parse_event_datetime(event_dt: &EventDateTime) -> Result<DateTime<Utc>> {
        if let Some(ref date_time_str) = event_dt.date_time {
            // Parse RFC3339 datetime
            DateTime::parse_from_rfc3339(date_time_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Failed to parse dateTime: {}", e))
        } else if let Some(ref date_str) = event_dt.date {
            // Parse date-only (all-day event)
            // Assume midnight UTC
            let naive_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Failed to parse date: {}", e))?;
            let naive_datetime = naive_date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Failed to create datetime from date"))?;
            Ok(DateTime::<Utc>::from_naive_utc_and_offset(
                naive_datetime,
                Utc,
            ))
        } else {
            Err(anyhow::anyhow!(
                "EventDateTime has neither dateTime nor date"
            ))
        }
    }

    /// Check connectivity to Google Calendar API
    async fn check_connectivity(api_client: &Arc<HiNotesClient>) -> bool {
        // Use the API client's token to check connectivity
        let token = match api_client.get_token().await {
            Some(t) => t,
            None => return false,
        };

        match reqwest::Client::new()
            .get(CONNECTIVITY_CHECK_URL)
            .bearer_auth(&token)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    true
                } else {
                    log::debug!(
                        "Google Calendar API connectivity check failed: {}",
                        response.status()
                    );
                    false
                }
            }
            Err(e) => {
                log::debug!("Google Calendar API connectivity check failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::HiNotesClient;
    use crate::db::Database;

    #[tokio::test]
    async fn test_calendar_sync_lifecycle() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let client = Arc::new(HiNotesClient::with_base_url(
            "https://hinotes.hidock.com/v1".to_string(),
        ));
        let sync = CalendarSync::new(client, db);

        // Initially not running
        assert!(!sync.is_running().await);

        // Set Google token
        sync.set_google_token("test-token".to_string()).await;

        // Start the worker
        sync.start_sync_loop().await.unwrap();
        assert!(sync.is_running().await);

        // Stop the worker
        sync.stop().await;
    }

    #[tokio::test]
    async fn test_set_calendar_id() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let client = Arc::new(HiNotesClient::with_base_url(
            "https://hinotes.hidock.com/v1".to_string(),
        ));
        let sync = CalendarSync::new(client, db);

        sync.set_calendar_id("user@example.com".to_string()).await;
        let id = sync.calendar_id.lock().await.clone();
        assert_eq!(id, "user@example.com");
    }

    #[test]
    fn test_parse_event_datetime_rfc3339() {
        let event_dt = EventDateTime {
            date_time: Some("2024-01-15T10:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };

        let result = CalendarSync::parse_event_datetime(&event_dt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_event_datetime_date_only() {
        let event_dt = EventDateTime {
            date_time: None,
            date: Some("2024-01-15".to_string()),
            time_zone: None,
        };

        let result = CalendarSync::parse_event_datetime(&event_dt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_event_datetime_missing() {
        let event_dt = EventDateTime {
            date_time: None,
            date: None,
            time_zone: None,
        };

        let result = CalendarSync::parse_event_datetime(&event_dt);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sync_now_without_token() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let client = Arc::new(HiNotesClient::with_base_url(
            "https://hinotes.hidock.com/v1".to_string(),
        ));
        let sync = CalendarSync::new(client, db);

        let result = sync.sync_now().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No Google token"));
    }
}
