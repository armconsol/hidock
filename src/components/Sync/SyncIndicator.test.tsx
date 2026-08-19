import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { SyncIndicator } from './SyncIndicator';
import { useSyncStore } from '../../store/syncStore';

describe('SyncIndicator', () => {
  beforeEach(() => {
    // Reset store
    const store = useSyncStore.getState();
    store.clearPendingOperations();
    store.setStatus('synced');
    store.setError(null);
  });

  it('should render synced status', () => {
    const store = useSyncStore.getState();
    store.setStatus('synced');

    render(<SyncIndicator />);
    expect(screen.getByText('Synced')).toBeInTheDocument();
  });

  it('should render syncing status', () => {
    const store = useSyncStore.getState();
    store.setStatus('syncing');

    render(<SyncIndicator />);
    expect(screen.getByText('Syncing...')).toBeInTheDocument();
  });

  it('should render offline status', () => {
    const store = useSyncStore.getState();
    store.setStatus('offline');

    render(<SyncIndicator />);
    expect(screen.getByText('Offline')).toBeInTheDocument();
  });

  it('should render failed status', () => {
    const store = useSyncStore.getState();
    store.setStatus('failed');

    render(<SyncIndicator />);
    expect(screen.getByText('Failed')).toBeInTheDocument();
  });

  it('should display pending operation count', () => {
    const store = useSyncStore.getState();
    store.addPendingOperation('create', 'note', 'note-1', {});
    store.addPendingOperation('update', 'note', 'note-2', {});

    render(<SyncIndicator />);
    expect(screen.getByText('(2)')).toBeInTheDocument();
  });

  it('should not display count when no pending operations', () => {
    const store = useSyncStore.getState();
    store.setStatus('synced');

    render(<SyncIndicator />);
    expect(screen.queryByText(/\(\d+\)/)).not.toBeInTheDocument();
  });
});
