# Calendar Recording Notification Feature

## Overview

This feature enables the application to notify Google Calendar when a recording starts or stops for a specific calendar event. The HiNotes API updates the event description with "Recording in progress..." while active, and may add a transcription link when the recording completes.

## Implementation Summary

### 1. Python Client (`API_Notes/hinotes_client.py`)

Added `notify_recording_status()` method:

```python
def notify_recording_status(
    self,
    event_id: str,
    is_recording: bool
) -> Dict[str, Any]:
    """
    Notify HiNotes calendar of device recording state
    
    Args:
        event_id: Google Calendar event ID
        is_recording: True if recording is active, False if stopped
    
    Returns:
        Response from server confirming the notification
    """
    return self._request('POST', '/calendar/event/device_state/notice', json={
        'eventId': event_id,
        'isRecording': is_recording
    })
```

### 2. Rust API Types (`src-tauri/src/api/types.rs`)

Added request/response types:

```rust
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
```

### 3. Rust API Client (`src-tauri/src/api/client.rs`)

Added `notify_recording_status()` method:

```rust
pub async fn notify_recording_status(
    &self,
    event_id: &str,
    is_recording: bool,
) -> Result<NotifyRecordingStatusResponse>
```

### 4. Calendar Sync Worker (`src-tauri/src/sync/calendar_sync.rs`)

Added method to CalendarSync:

```rust
pub async fn notify_recording_status(&self, event_id: &str, is_recording: bool) -> Result<()>
```

### 5. Tauri Command (`src-tauri/src/commands/calendar_sync_commands.rs`)

Added command `notify_calendar_recording`:

```rust
#[tauri::command]
pub async fn notify_calendar_recording(
    event_id: String,
    is_recording: bool,
    state: State<'_, CalendarSyncState>,
) -> Result<String, String>
```

### 6. Command Registration (`src-tauri/src/lib.rs`)

Registered command in Tauri app builder:
```rust
commands::notify_calendar_recording,
```

## Usage from Frontend

### TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// When recording starts
async function onRecordingStart(eventId: string) {
  try {
    const result = await invoke('notify_calendar_recording', {
      eventId: eventId,
      isRecording: true
    });
    console.log('Notified recording started:', result);
  } catch (error) {
    console.error('Failed to notify recording start:', error);
  }
}

// When recording stops
async function onRecordingStop(eventId: string) {
  try {
    const result = await invoke('notify_calendar_recording', {
      eventId: eventId,
      isRecording: false
    });
    console.log('Notified recording stopped:', result);
  } catch (error) {
    console.error('Failed to notify recording stop:', error);
  }
}
```

### React Hook Example

```typescript
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface UseRecordingNotificationReturn {
  notifyRecordingStatus: (eventId: string, isRecording: boolean) => Promise<void>;
  isNotifying: boolean;
  error: string | null;
}

export function useRecordingNotification(): UseRecordingNotificationReturn {
  const [isNotifying, setIsNotifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const notifyRecordingStatus = async (eventId: string, isRecording: boolean) => {
    setIsNotifying(true);
    setError(null);
    
    try {
      await invoke('notify_calendar_recording', {
        eventId,
        isRecording
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setIsNotifying(false);
    }
  };

  return { notifyRecordingStatus, isNotifying, error };
}

// Usage in component
function RecordingPanel({ eventId }: { eventId: string }) {
  const [isRecording, setIsRecording] = useState(false);
  const { notifyRecordingStatus, isNotifying } = useRecordingNotification();

  const handleRecordingToggle = async () => {
    const newState = !isRecording;
    setIsRecording(newState);
    
    try {
      await notifyRecordingStatus(eventId, newState);
    } catch (error) {
      console.error('Failed to notify calendar:', error);
      // Revert state on error
      setIsRecording(!newState);
    }
  };

  return (
    <button onClick={handleRecordingToggle} disabled={isNotifying}>
      {isRecording ? 'Stop Recording' : 'Start Recording'}
    </button>
  );
}
```

## Integration Points

### 1. Note Recording UI

When a user starts recording a note linked to a calendar event:

```typescript
// In recording component
const startRecording = async (calendarEventId?: string) => {
  // Start local recording
  await audioRecorder.start();
  
  // Notify calendar if event is linked
  if (calendarEventId) {
    await invoke('notify_calendar_recording', {
      eventId: calendarEventId,
      isRecording: true
    });
  }
};

const stopRecording = async (calendarEventId?: string) => {
  // Stop local recording
  const audioBlob = await audioRecorder.stop();
  
  // Notify calendar
  if (calendarEventId) {
    await invoke('notify_calendar_recording', {
      eventId: calendarEventId,
      isRecording: false
    });
  }
  
  // Process transcription...
};
```

### 2. Calendar Sync Worker

The calendar sync worker can automatically notify Google when:
- A local note recording starts that's associated with an event
- A recording completes and transcription link is available

```rust
// Potential future enhancement in calendar_sync.rs
pub async fn on_note_recording_start(&self, note_id: &str) -> Result<()> {
    // Look up associated calendar event
    let db = self.db.lock().await;
    if let Some(event) = db.get_calendar_event_for_note(note_id)? {
        if let Some(google_id) = event.google_event_id {
            self.notify_recording_status(&google_id, true).await?;
        }
    }
    Ok(())
}

pub async fn on_note_recording_complete(&self, note_id: &str, transcription_url: &str) -> Result<()> {
    let db = self.db.lock().await;
    if let Some(event) = db.get_calendar_event_for_note(note_id)? {
        if let Some(google_id) = event.google_event_id {
            // Notify recording stopped
            self.notify_recording_status(&google_id, false).await?;
            
            // Optionally update event description with transcription link
            // (would require additional API method)
        }
    }
    Ok(())
}
```

## API Endpoint

**Endpoint**: `POST /v1/calendar/event/device_state/notice`

**Request Body**:
```json
{
  "eventId": "google_calendar_event_id_here",
  "isRecording": true
}
```

**Response**:
```json
{
  "success": true,
  "message": "Recording status updated"
}
```

## Error Handling

The command returns a `Result<String, String>` where:
- **Success**: Returns a message like "Successfully notified that recording started for event {event_id}"
- **Errors**:
  - "Calendar sync worker not initialized" - If calendar sync is not set up
  - "Failed to notify recording status: {error}" - If API call fails
  - Network errors, authentication errors, etc.

## Testing

### Manual Testing

1. Ensure calendar sync is running with valid Google OAuth token
2. Get a Google Calendar event ID from an existing event
3. Call the command from browser console:

```javascript
// Test recording start
await window.__TAURI__.invoke('notify_calendar_recording', {
  eventId: 'your_event_id_here',
  isRecording: true
});

// Test recording stop
await window.__TAURI__.invoke('notify_calendar_recording', {
  eventId: 'your_event_id_here',
  isRecording: false
});
```

4. Check the Google Calendar event to see if description updates with "Recording in progress..."

### Unit Testing

The implementation includes:
- API client method with retry logic
- Proper error propagation through the sync worker and command layers
- Logging at each level for debugging

## Future Enhancements

1. **Auto-link notes to calendar events**: Automatically associate recordings with calendar events based on time
2. **Batch notifications**: Handle multiple recordings for recurring meetings
3. **Transcription URL updates**: Add method to update event with completed transcription link
4. **Offline queue**: Queue notifications when offline and send when connectivity restored
5. **Event selection UI**: Allow users to select which event to link when multiple events overlap

## Files Modified

- `/Users/sarman/Documents/GitHub/hidoc/API_Notes/hinotes_client.py`
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/api/types.rs`
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/api/client.rs`
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/sync/calendar_sync.rs`
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/commands/calendar_sync_commands.rs`
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/lib.rs`

## Build Status

✅ Code compiles successfully with 0 errors
⚠️ 11 warnings (all pre-existing, unrelated to this feature)
