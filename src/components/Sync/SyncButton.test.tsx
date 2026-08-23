import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { SyncButton } from './SyncButton';
import { useSyncStore } from '../../store/syncStore';

// Mock Ant Design message component
vi.mock('antd', async () => {
  const actual = await vi.importActual('antd');
  return {
    ...actual,
    message: {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    },
  };
});

describe('SyncButton', () => {
  beforeEach(() => {
    // Reset store
    const store = useSyncStore.getState();
    store.clearPendingOperations();
    store.setStatus('synced');
    store.setError(null);
    store.setOnline(true);
  });

  it('should render sync button', () => {
    render(<SyncButton />);
    expect(screen.getByRole('button', { name: /sync/i })).toBeInTheDocument();
  });

  it('should display pending count in button text', () => {
    const store = useSyncStore.getState();
    store.addPendingOperation('create', 'note', 'note-1', {});
    store.addPendingOperation('update', 'note', 'note-2', {});

    render(<SyncButton />);
    expect(screen.getByText(/Sync \(2\)/)).toBeInTheDocument();
  });

  it('should be disabled when offline', () => {
    const store = useSyncStore.getState();
    store.setOnline(false);

    render(<SyncButton />);
    const button = screen.getByRole('button', { name: /sync/i });
    expect(button).toBeDisabled();
  });

  it('should show loading state when syncing', () => {
    const store = useSyncStore.getState();
    store.setStatus('syncing');

    render(<SyncButton />);
    const button = screen.getByRole('button', { name: /sync/i });
    expect(button).toHaveClass('ant-btn-loading');
  });

  it('should trigger sync on button click', async () => {
    const store = useSyncStore.getState();
    const triggerSyncSpy = vi.spyOn(store, 'triggerSync');

    render(<SyncButton />);
    const button = screen.getByRole('button', { name: /sync/i });

    fireEvent.click(button);

    await waitFor(() => {
      expect(triggerSyncSpy).toHaveBeenCalled();
    });
  });

  it('should render with pending operations', () => {
    const store = useSyncStore.getState();
    store.addPendingOperation('create', 'note', 'note-1', { title: 'Test' });

    render(<SyncButton />);

    // Verify button is rendered with count
    expect(screen.getByText(/Sync \(1\)/)).toBeInTheDocument();
  });

  it('should render without pending operations', () => {
    render(<SyncButton />);

    // Verify button is rendered without count
    const button = screen.getByRole('button', { name: /sync/i });
    expect(button).toBeInTheDocument();
    expect(screen.queryByText(/\(\d+\)/)).not.toBeInTheDocument();
  });

  it('should render with error state', () => {
    const store = useSyncStore.getState();
    store.setError('Network error');

    render(<SyncButton />);

    // Verify button still renders with error in store
    expect(screen.getByRole('button', { name: /sync/i })).toBeInTheDocument();
  });
});
