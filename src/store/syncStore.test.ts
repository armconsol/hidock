import { describe, it, expect, beforeEach } from 'vitest';
import { useSyncStore } from './syncStore';

describe('useSyncStore', () => {
  beforeEach(() => {
    // Reset store before each test
    const store = useSyncStore.getState();
    store.clearPendingOperations();
    store.setStatus('offline');
    store.setError(null);
  });

  describe('setStatus', () => {
    it('should update status', () => {
      const store = useSyncStore.getState();
      store.setStatus('syncing');
      expect(useSyncStore.getState().status).toBe('syncing');
    });
  });

  describe('setOnline', () => {
    it('should update online status', () => {
      const store = useSyncStore.getState();
      store.setOnline(true);
      expect(useSyncStore.getState().isOnline).toBe(true);
    });

    it('should set status to offline when going offline', () => {
      const store = useSyncStore.getState();
      store.setOnline(false);
      expect(useSyncStore.getState().status).toBe('offline');
    });
  });

  describe('addPendingOperation', () => {
    it('should add a new pending operation', () => {
      const store = useSyncStore.getState();
      store.addPendingOperation('create', 'note', 'note-123', { title: 'Test' });

      const operations = useSyncStore.getState().pendingOperations;
      expect(operations.length).toBe(1);
      expect(operations[0].type).toBe('create');
      expect(operations[0].entityType).toBe('note');
      expect(operations[0].entityId).toBe('note-123');
      expect(operations[0].retryCount).toBe(0);
    });

    it('should generate unique IDs for operations', () => {
      const store = useSyncStore.getState();
      store.addPendingOperation('create', 'note', 'note-1', {});
      store.addPendingOperation('update', 'note', 'note-2', {});

      const operations = useSyncStore.getState().pendingOperations;
      expect(operations[0].id).not.toBe(operations[1].id);
    });
  });

  describe('removePendingOperation', () => {
    it('should remove operation by ID', () => {
      const store = useSyncStore.getState();
      store.addPendingOperation('create', 'note', 'note-1', {});
      const operationId = useSyncStore.getState().pendingOperations[0].id;

      store.removePendingOperation(operationId);
      expect(useSyncStore.getState().pendingOperations.length).toBe(0);
    });

    it('should not affect other operations', () => {
      const store = useSyncStore.getState();
      store.addPendingOperation('create', 'note', 'note-1', {});
      store.addPendingOperation('update', 'note', 'note-2', {});

      const firstId = useSyncStore.getState().pendingOperations[0].id;
      store.removePendingOperation(firstId);

      const operations = useSyncStore.getState().pendingOperations;
      expect(operations.length).toBe(1);
      expect(operations[0].entityId).toBe('note-2');
    });
  });

  describe('incrementRetryCount', () => {
    it('should increment retry count for specific operation', () => {
      const store = useSyncStore.getState();
      store.addPendingOperation('create', 'note', 'note-1', {});
      const operationId = useSyncStore.getState().pendingOperations[0].id;

      store.incrementRetryCount(operationId);
      expect(useSyncStore.getState().pendingOperations[0].retryCount).toBe(1);

      store.incrementRetryCount(operationId);
      expect(useSyncStore.getState().pendingOperations[0].retryCount).toBe(2);
    });
  });

  describe('clearPendingOperations', () => {
    it('should clear all pending operations', () => {
      const store = useSyncStore.getState();
      store.addPendingOperation('create', 'note', 'note-1', {});
      store.addPendingOperation('update', 'note', 'note-2', {});
      store.addPendingOperation('delete', 'todo', 'todo-1', {});

      store.clearPendingOperations();
      expect(useSyncStore.getState().pendingOperations.length).toBe(0);
    });
  });

  describe('updateLastSyncTime', () => {
    it('should update last sync time to current time', () => {
      const store = useSyncStore.getState();
      const beforeTime = new Date();

      store.updateLastSyncTime();

      const lastSyncTime = useSyncStore.getState().lastSyncTime;
      expect(lastSyncTime).toBeInstanceOf(Date);
      expect(lastSyncTime!.getTime()).toBeGreaterThanOrEqual(beforeTime.getTime());
    });
  });

  describe('getPendingCount', () => {
    it('should return correct pending count', () => {
      const store = useSyncStore.getState();
      expect(store.getPendingCount()).toBe(0);

      store.addPendingOperation('create', 'note', 'note-1', {});
      expect(store.getPendingCount()).toBe(1);

      store.addPendingOperation('update', 'note', 'note-2', {});
      expect(store.getPendingCount()).toBe(2);
    });
  });

  describe('triggerSync', () => {
    it('should set status to offline when not online', async () => {
      const store = useSyncStore.getState();
      store.setOnline(false);
      store.addPendingOperation('create', 'note', 'note-1', {});

      await store.triggerSync();

      expect(useSyncStore.getState().status).toBe('offline');
    });

    it('should set status to synced when no pending operations', async () => {
      const store = useSyncStore.getState();
      store.setOnline(true);

      await store.triggerSync();

      expect(useSyncStore.getState().status).toBe('synced');
      expect(useSyncStore.getState().error).toBeNull();
    });

    it('should set status to syncing while processing', async () => {
      const store = useSyncStore.getState();
      store.setOnline(true);
      store.addPendingOperation('create', 'note', 'note-1', {});

      const syncPromise = store.triggerSync();

      // Check status during sync (before promise resolves)
      expect(useSyncStore.getState().status).toBe('syncing');

      await syncPromise;
    });

    it('should remove operations after successful sync', async () => {
      const store = useSyncStore.getState();
      store.setOnline(true);
      store.addPendingOperation('create', 'note', 'note-1', {});
      store.addPendingOperation('update', 'note', 'note-2', {});

      await store.triggerSync();

      expect(useSyncStore.getState().pendingOperations.length).toBe(0);
      expect(useSyncStore.getState().status).toBe('synced');
    });

    it('should update last sync time after successful sync', async () => {
      const store = useSyncStore.getState();
      store.setOnline(true);
      store.addPendingOperation('create', 'note', 'note-1', {});

      const beforeTime = new Date();
      await store.triggerSync();

      const lastSyncTime = useSyncStore.getState().lastSyncTime;
      expect(lastSyncTime).toBeInstanceOf(Date);
      expect(lastSyncTime!.getTime()).toBeGreaterThanOrEqual(beforeTime.getTime());
    });
  });

  describe('setError', () => {
    it('should set and clear error messages', () => {
      const store = useSyncStore.getState();

      store.setError('Test error');
      expect(useSyncStore.getState().error).toBe('Test error');

      store.setError(null);
      expect(useSyncStore.getState().error).toBeNull();
    });
  });
});
