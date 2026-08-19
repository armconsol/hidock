# Calendar Bidirectional Sync Implementation

## Overview
Implemented bidirectional synchronization between local database and Google Calendar with last-write-wins conflict resolution.

## Changes Made

### 1. Database Schema Updates (`src-tauri/src/db/schema.sql`)
Added new fields to `calendar_events` table:
- `updated_at DATETIME NOT NULL` - Tracks when event was last modified locally
- `google_event_id TEXT` - Maps local event to Google Calendar event ID
- `sync_status TEXT` - Tracks sync state (synced, pending_push, conflict)

### 2. Type System Updates

#### `src-tauri/src/db/types.rs`
- Updated `CalendarEvent` struct with new fields
- Added `SyncStatus` enum with three states:
  - `Synced` - Event is in sync with Google
  - `PendingPush` - Local changes need to be pushed to Google
  - `Conflict` - Detected conflict between local and remote

#### `src-tauri/src/api/types.rs`
- Added `UpdateEventRequest` struct for updating Google Calendar events
- Added `updated` field to `GoogleCalendarEvent` to track remote modification time

### 3. API Client Updates (`src-tauri/src/api/client.rs`)
Added two new methods for Google Calendar API:
- `update_event()` - Update an existing calendar event
- `delete_event()` - Delete a calendar event

### 4. Database Operations (`src-tauri/src/db/mod.rs`)

#### Updated Existing Methods
- `insert_calendar_event()` - Now includes new fields
- `get_calendar_event()` - Returns events with new fields
- `list_calendar_events()` - Includes new fields in results
- `list_calendar_events_in_range()` - Includes new fields in results
- `update_calendar_event()` - Updates all fields including sync status
- `mark_calendar_event_synced()` - Also updates sync_status to Synced

#### New Methods
- `get_pending_push_events()` - Get all events with PendingPush status
- `update_calendar_event_sync_status()` - Update just the sync status
- `get_calendar_event_by_google_id()` - Find local event by Google Calendar ID

### 5. Sync Engine (`src-tauri/src/sync/calendar_sync.rs`)

#### Bidirectional Sync Flow
The `sync_events_internal()` method now implements a three-phase sync:

**Phase 1: Push Local Changes**
- Query events with `PendingPush` status
- For new local events (no google_event_id):
  - Create in Google Calendar
  - Store returned Google ID
  - Mark as Synced
- For modified local events (has google_event_id):
  - Update in Google Calendar
  - Mark as Synced

**Phase 2: Fetch Remote Events**
- Pull events from Google Calendar within sync window (7 days past, 30 days future)

**Phase 3: Pull Remote Changes**
- For each remote event:
  - Check if exists locally by google_event_id
  - If new: Insert as local event
  - If exists and Synced: Update if changed
  - If exists and PendingPush: Resolve conflict

#### Conflict Resolution (Last-Write-Wins)
When a conflict is detected (local PendingPush + remote changes):
1. Compare `updated_at` timestamps (local vs remote)
2. If local is newer: Keep local version, will push in next sync
3. If remote is newer: Accept remote version, overwrite local
4. Log conflict for user review

#### Helper Methods
- `push_local_changes()` - Coordinate pushing pending local events
- `push_event_to_google()` - Push single event (create or update)
- `pull_remote_changes()` - Coordinate pulling remote events
- `sync_remote_event()` - Sync single remote event with conflict handling
- `resolve_conflict()` - Implement last-write-wins logic
- `datetime_to_event_datetime()` - Convert between timestamp formats

## Usage

### Initial Sync on Login
```rust
// Set Google OAuth token
calendar_sync.set_google_token(google_token).await;

// Start background sync loop (syncs every 30 seconds)
calendar_sync.start_sync_loop().await?;
```

### Manual Sync Trigger
```rust
let synced_count = calendar_sync.sync_now().await?;
println!("Synced {} events", synced_count);
```

### Creating Local Event
```rust
let event = CalendarEvent {
    id: uuid::Uuid::new_v4().to_string(),
    title: "New Meeting".to_string(),
    start_time: Utc::now(),
    end_time: Utc::now() + Duration::hours(1),
    source: EventSource::Hinotes,
    meeting_url: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    synced_at: None,
    google_event_id: None,
    sync_status: SyncStatus::PendingPush, // Will be pushed on next sync
};

db.insert_calendar_event(&event)?;
```

### Updating Local Event
```rust
let mut event = db.get_calendar_event(id)?.unwrap();
event.title = "Updated Title".to_string();
event.updated_at = Utc::now();
event.sync_status = SyncStatus::PendingPush;
db.update_calendar_event(&event)?;
```

## Sync Behavior

### Automatic Sync
- Runs every 30 seconds (configurable)
- Only syncs if Google token is available
- Checks Google Calendar API connectivity before syncing
- Uses exponential backoff for retries

### Sync Window
- Past: 7 days
- Future: 30 days
- Events outside this window are not synced

### Conflict Scenarios

| Local State | Remote State | Resolution |
|-------------|--------------|------------|
| New event | Doesn't exist | Push to Google |
| Modified (PendingPush) | Modified | Compare timestamps, keep newest |
| Synced | Modified | Accept remote changes |
| Synced | Deleted | Keep local (Google deletes not yet handled) |
| Modified (PendingPush) | Doesn't exist | Push to Google |

## Testing Needs

The following should be tested:
1. Initial sync from Google Calendar (pull)
2. Creating new local events and syncing (push)
3. Updating existing events locally (push)
4. Concurrent modifications (conflict resolution)
5. Sync with network interruption (retry logic)
6. Large event lists (performance)
7. Events with different timezones
8. All-day events vs timed events

## Future Enhancements

1. **Delete Handling**: Currently, deletions are not synced. Need to:
   - Track deleted events
   - Handle remote deletions
   - Implement soft delete with tombstone records

2. **Conflict Logging**: 
   - Store conflict history for user review
   - UI to show conflicts and let user choose resolution

3. **Selective Sync**:
   - Allow users to choose which calendars to sync
   - Filter by event type/category

4. **Offline Queue**:
   - Better handling of offline mode
   - Queue operations when network unavailable

5. **Incremental Sync**:
   - Use Google Calendar sync tokens
   - Only fetch changes since last sync
   - Reduce API calls and bandwidth

6. **Recurring Events**:
   - Handle recurring event series
   - Sync recurrence rules

7. **Attendees & Reminders**:
   - Sync event attendees
   - Sync reminders and notifications

## Migration Notes

Existing databases will need migration to add the new fields:
```sql
ALTER TABLE calendar_events ADD COLUMN updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE calendar_events ADD COLUMN google_event_id TEXT;
ALTER TABLE calendar_events ADD COLUMN sync_status TEXT CHECK(sync_status IN ('synced', 'pending_push', 'conflict')) NOT NULL DEFAULT 'synced';
```

Existing events should be marked as Synced and updated_at should be set to created_at or synced_at.
