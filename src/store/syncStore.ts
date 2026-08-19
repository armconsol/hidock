import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { SyncStatus, SyncOperation, SyncOperationType } from '../types/sync';

interface SyncStore {
  status: SyncStatus;
  lastSyncTime: Date | null;
  pendingOperations: SyncOperation[];
  error: string | null;
  isOnline: boolean;

  // Actions
  setStatus: (status: SyncStatus) => void;
  setOnline: (isOnline: boolean) => void;
  setError: (error: string | null) => void;
  addPendingOperation: (
    type: SyncOperationType,
    entityType: 'note' | 'todo' | 'folder' | 'template',
    entityId: string,
    data: unknown
  ) => void;
  removePendingOperation: (id: string) => void;
  incrementRetryCount: (id: string) => void;
  clearPendingOperations: () => void;
  updateLastSyncTime: () => void;
  triggerSync: () => Promise<void>;
  getPendingCount: () => number;
}

export const useSyncStore = create<SyncStore>()(
  persist(
    (set, get) => ({
      status: 'offline',
      lastSyncTime: null,
      pendingOperations: [],
      error: null,
      isOnline: navigator.onLine,

      setStatus: (status) => {
        set({ status });
      },

      setOnline: (isOnline) => {
        set({ isOnline });
        if (isOnline) {
          // Auto-trigger sync when coming online
          get().triggerSync();
        } else {
          set({ status: 'offline' });
        }
      },

      setError: (error) => {
        set({ error });
      },

      addPendingOperation: (type, entityType, entityId, data) => {
        const operation: SyncOperation = {
          id: crypto.randomUUID(),
          type,
          entityType,
          entityId,
          data,
          timestamp: new Date(),
          retryCount: 0,
        };

        set((state) => ({
          pendingOperations: [...state.pendingOperations, operation],
        }));

        // Trigger sync if online
        if (get().isOnline) {
          get().triggerSync();
        }
      },

      removePendingOperation: (id) => {
        set((state) => ({
          pendingOperations: state.pendingOperations.filter((op) => op.id !== id),
        }));
      },

      incrementRetryCount: (id) => {
        set((state) => ({
          pendingOperations: state.pendingOperations.map((op) =>
            op.id === id ? { ...op, retryCount: op.retryCount + 1 } : op
          ),
        }));
      },

      clearPendingOperations: () => {
        set({ pendingOperations: [] });
      },

      updateLastSyncTime: () => {
        set({ lastSyncTime: new Date() });
      },

      triggerSync: async () => {
        const state = get();

        if (!state.isOnline) {
          set({ status: 'offline' });
          return;
        }

        if (state.pendingOperations.length === 0) {
          set({ status: 'synced', error: null });
          return;
        }

        set({ status: 'syncing', error: null });

        try {
          // Process pending operations
          for (const operation of state.pendingOperations) {
            try {
              // TODO: Implement actual API calls here
              // await syncOperationToServer(operation);

              // Simulate sync delay
              await new Promise((resolve) => setTimeout(resolve, 100));

              // Remove successful operation
              get().removePendingOperation(operation.id);
            } catch (error) {
              const errorMessage = error instanceof Error ? error.message : 'Sync failed';

              // Increment retry count
              get().incrementRetryCount(operation.id);

              // Mark as failed if too many retries
              if (operation.retryCount >= 3) {
                set({
                  status: 'failed',
                  error: `Failed to sync after ${operation.retryCount} attempts: ${errorMessage}`,
                });
                return;
              }
            }
          }

          // Update sync status
          set({
            status: 'synced',
            lastSyncTime: new Date(),
            error: null,
          });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Unknown error';
          set({
            status: 'failed',
            error: errorMessage,
          });
        }
      },

      getPendingCount: () => {
        return get().pendingOperations.length;
      },
    }),
    {
      name: 'sync-storage',
      partialize: (state) => ({
        pendingOperations: state.pendingOperations,
        lastSyncTime: state.lastSyncTime,
      }),
    }
  )
);

// Set up online/offline listeners
if (typeof window !== 'undefined') {
  window.addEventListener('online', () => {
    useSyncStore.getState().setOnline(true);
  });

  window.addEventListener('offline', () => {
    useSyncStore.getState().setOnline(false);
  });
}
