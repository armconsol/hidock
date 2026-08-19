// Sync types

export type SyncStatus = 'synced' | 'syncing' | 'offline' | 'failed';

export type SyncOperationType = 'create' | 'update' | 'delete';

export interface SyncOperation {
  id: string;
  type: SyncOperationType;
  entityType: 'note' | 'todo' | 'folder' | 'template';
  entityId: string;
  data: unknown;
  timestamp: Date;
  retryCount: number;
  error?: string;
}

export interface SyncState {
  status: SyncStatus;
  lastSyncTime: Date | null;
  pendingOperations: SyncOperation[];
  error: string | null;
  isOnline: boolean;
}
