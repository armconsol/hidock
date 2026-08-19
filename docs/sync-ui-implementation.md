# Sync Status UI Implementation

## Overview

This document describes the implementation of the sync status UI components for the HiNotes Desktop application. The sync system provides real-time feedback about synchronization state, pending operations, and network connectivity.

## Architecture

### Components

1. **SyncIndicator** - Status badge displaying current sync state
2. **SyncButton** - Manual sync trigger with pending operations view
3. **syncStore** - Zustand store managing sync state

### Data Flow

```
User Action → Store Action → State Update → Component Re-render → UI Update
                    ↓
              API Sync (when online)
```

## File Structure

```
src/
├── components/
│   └── Sync/
│       ├── SyncIndicator.tsx       # Status badge component
│       ├── SyncIndicator.css       # Indicator styles
│       ├── SyncIndicator.test.tsx  # Component tests
│       ├── SyncButton.tsx          # Manual sync button
│       ├── SyncButton.css          # Button styles
│       ├── SyncButton.test.tsx     # Component tests
│       └── index.ts                # Exports
├── store/
│   ├── syncStore.ts                # Sync state management
│   └── syncStore.test.ts           # Store tests
├── types/
│   └── sync.ts                     # TypeScript types
└── pages/
    └── SyncDemo.tsx                # Demo page
```

## Type Definitions

### SyncStatus

```typescript
type SyncStatus = 'synced' | 'syncing' | 'offline' | 'failed';
```

- **synced** - All operations successfully synced
- **syncing** - Currently processing pending operations
- **offline** - No network connection
- **failed** - Sync operation failed

### SyncOperation

```typescript
interface SyncOperation {
  id: string;
  type: 'create' | 'update' | 'delete';
  entityType: 'note' | 'todo' | 'folder' | 'template';
  entityId: string;
  data: unknown;
  timestamp: Date;
  retryCount: number;
  error?: string;
}
```

## Store API

### State

```typescript
interface SyncStore {
  status: SyncStatus;
  lastSyncTime: Date | null;
  pendingOperations: SyncOperation[];
  error: string | null;
  isOnline: boolean;
}
```

### Actions

#### setStatus(status: SyncStatus)
Updates the current sync status.

```typescript
const { setStatus } = useSyncStore();
setStatus('syncing');
```

#### setOnline(isOnline: boolean)
Updates online status and triggers sync when coming online.

```typescript
const { setOnline } = useSyncStore();
setOnline(true);
```

#### addPendingOperation(type, entityType, entityId, data)
Adds a new operation to the pending queue.

```typescript
const { addPendingOperation } = useSyncStore();
addPendingOperation('create', 'note', 'note-123', {
  title: 'New Note',
  content: 'Note content'
});
```

#### removePendingOperation(id: string)
Removes a specific operation from the queue.

```typescript
const { removePendingOperation } = useSyncStore();
removePendingOperation('operation-id');
```

#### triggerSync()
Manually triggers synchronization process.

```typescript
const { triggerSync } = useSyncStore();
await triggerSync();
```

#### clearPendingOperations()
Clears all pending operations.

```typescript
const { clearPendingOperations } = useSyncStore();
clearPendingOperations();
```

#### getPendingCount()
Returns the number of pending operations.

```typescript
const { getPendingCount } = useSyncStore();
const count = getPendingCount();
```

## Component Usage

### SyncIndicator

Displays current sync status with visual feedback.

```tsx
import { SyncIndicator } from '@/components/Sync';

function Header() {
  return (
    <div className="header">
      <SyncIndicator />
    </div>
  );
}
```

**Features:**
- Color-coded status badge (green/blue/gray/red)
- Spinning icon during sync
- Tooltip with detailed information
- Pending operation count badge
- Last sync time display

### SyncButton

Manual sync trigger with pending operations popover.

```tsx
import { SyncButton } from '@/components/Sync';

function Toolbar() {
  return (
    <div className="toolbar">
      <SyncButton />
    </div>
  );
}
```

**Features:**
- Manual sync trigger
- Loading state during sync
- Disabled when offline
- Popover showing pending operations
- Clear all pending operations
- Error message display

## Integration Guide

### Step 1: Add to Layout

Add sync components to your app layout header:

```tsx
import { SyncIndicator, SyncButton } from '@/components/Sync';

export function AppLayout() {
  return (
    <div className="app-layout">
      <header>
        <div className="header-left">
          {/* Logo, navigation, etc. */}
        </div>
        <div className="header-right">
          <SyncIndicator />
          <SyncButton />
        </div>
      </header>
      {/* Main content */}
    </div>
  );
}
```

### Step 2: Queue Operations

When users create, update, or delete data:

```tsx
import { useSyncStore } from '@/store/syncStore';
import { useNotesStore } from '@/store/notesStore';

function NoteEditor() {
  const { addNote } = useNotesStore();
  const { addPendingOperation } = useSyncStore();

  const handleSave = (note) => {
    // Save locally
    addNote(note);

    // Queue for sync
    addPendingOperation('create', 'note', note.id, note);
  };

  return (
    <form onSubmit={handleSave}>
      {/* Form fields */}
    </form>
  );
}
```

### Step 3: Implement Sync Logic

Modify `syncStore.ts` to call actual API endpoints:

```typescript
triggerSync: async () => {
  const state = get();

  for (const operation of state.pendingOperations) {
    try {
      // Replace with actual API calls
      switch (operation.type) {
        case 'create':
          await api.create(operation.entityType, operation.data);
          break;
        case 'update':
          await api.update(operation.entityType, operation.entityId, operation.data);
          break;
        case 'delete':
          await api.delete(operation.entityType, operation.entityId);
          break;
      }

      get().removePendingOperation(operation.id);
    } catch (error) {
      // Handle error
    }
  }
},
```

## Testing

### Running Tests

```bash
# Run all tests
npm test

# Run specific test file
npm test syncStore.test.ts

# Run with UI
npm run test:ui
```

### Test Coverage

- **syncStore.test.ts** - 100% coverage of store actions
- **SyncIndicator.test.tsx** - Component rendering for all states
- **SyncButton.test.tsx** - User interactions and popover behavior

## Demo Page

Visit `/sync-demo` to see the sync UI in action with interactive controls.

**Features:**
- Live state visualization
- Manual state control
- Add test operations
- Trigger sync manually
- View pending operations list

## Styling

### CSS Variables Used

```css
--color-fill-2        /* Hover background */
--color-text-3        /* Secondary text */
--color-text-4        /* Disabled text */
--color-border-2      /* Borders */
--color-success-6     /* Success state */
--color-primary-6     /* Syncing state */
--color-danger-6      /* Error state */
--color-danger-light-1 /* Error background */
```

### Customization

Override styles in your app's CSS:

```css
.sync-indicator {
  padding: 6px 14px;
  border-radius: 6px;
}

.sync-indicator-text {
  font-size: 14px;
}
```

## Performance Considerations

### Store Persistence

The store persists:
- Pending operations (critical for offline support)
- Last sync time

Does NOT persist:
- Current status (recalculated on load)
- Error messages (transient)
- Online status (detected from browser)

### Automatic Sync Triggers

Sync is automatically triggered when:
1. Network connection is restored
2. New operation is added (if online)
3. User manually clicks sync button

### Retry Logic

Failed operations are automatically retried:
- Max retry count: 3 attempts
- Retry count incremented on each failure
- Status changes to 'failed' after max retries

## Best Practices

### 1. Always Queue Operations

```typescript
// ✅ Good - Queue for sync
addNote(note);
addPendingOperation('create', 'note', note.id, note);

// ❌ Bad - Direct API call
await api.createNote(note);
```

### 2. Handle Offline Gracefully

```typescript
if (!isOnline) {
  Message.info('Changes will sync when you\'re back online');
}
```

### 3. Provide Feedback

```typescript
// Show loading state
setStatus('syncing');

// Show success
setStatus('synced');
Message.success('Synced successfully');
```

### 4. Clean Up on Success

```typescript
// Remove operation after successful sync
removePendingOperation(operation.id);
```

## Troubleshooting

### Sync Not Triggering

**Check:**
- `isOnline` is `true`
- Pending operations exist
- No errors in browser console

### Operations Not Clearing

**Check:**
- API endpoints are working
- `removePendingOperation()` is called after success
- No exceptions during sync

### Status Stuck on "Syncing"

**Check:**
- All operations complete successfully
- No infinite loops in sync logic
- Status is updated after completion

## Future Enhancements

### Planned Features

1. **Conflict Resolution**
   - Detect server-side changes
   - Merge strategies for conflicts
   - User prompt for manual resolution

2. **Batch Sync**
   - Group operations by entity type
   - Optimize API calls
   - Progress indicator for large batches

3. **Background Sync**
   - Use Web Workers for sync
   - Service Worker integration
   - Push notifications on completion

4. **Sync History**
   - Log all sync operations
   - View past syncs
   - Rollback capability

5. **Advanced Retry**
   - Exponential backoff
   - Circuit breaker pattern
   - Priority queue for operations

## API Integration

### Expected Backend Endpoints

```typescript
// Create
POST /v1/{entityType}
Body: { ...data }
Response: { id, ...data }

// Update
PUT /v1/{entityType}/{id}
Body: { ...updates }
Response: { id, ...data }

// Delete
DELETE /v1/{entityType}/{id}
Response: { success: true }
```

### Error Handling

```typescript
try {
  await api.sync(operation);
} catch (error) {
  if (error.status === 409) {
    // Conflict - needs resolution
  } else if (error.status === 401) {
    // Unauthorized - re-authenticate
  } else {
    // Other errors - retry
  }
}
```

## License

This implementation is part of the HiNotes Desktop project and follows the same license.
