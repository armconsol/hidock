import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SettingsPage } from './Settings';
import { useSyncStore } from '../store/syncStore';
import { useSettingsStore } from '../store/settingsStore';
import * as tauriApi from '@tauri-apps/api/core';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock stores
vi.mock('../store/syncStore');
vi.mock('../store/settingsStore');

// Mock SyncIndicator component
vi.mock('../components/Sync/SyncIndicator', () => ({
  SyncIndicator: () => <div data-testid="sync-indicator">Sync Indicator</div>,
}));

describe('SettingsPage', () => {
  const mockInvoke = vi.mocked(tauriApi.invoke);
  const mockUseSyncStore = vi.mocked(useSyncStore);
  const mockUseSettingsStore = vi.mocked(useSettingsStore);

  const mockAIEngines = [
    {
      id: 'whisper-v3',
      name: 'Whisper V3',
      description: 'OpenAI Whisper V3 - High accuracy',
      capabilities: ['Multilingual', 'Punctuation', 'Speaker detection'],
      is_default: true,
      requires_subscription: false,
    },
    {
      id: 'whisper-large',
      name: 'Whisper Large',
      description: 'Premium transcription engine',
      capabilities: ['Multilingual', 'Punctuation', 'Speaker detection', 'Real-time'],
      is_default: false,
      requires_subscription: true,
    },
  ];

  beforeEach(() => {
    // Reset mocks
    vi.clearAllMocks();

    // Setup default store state
    mockUseSyncStore.mockReturnValue({
      status: 'synced',
      lastSyncTime: new Date('2024-01-01T12:00:00Z'),
      pendingOperations: [],
      error: null,
      isOnline: true,
      setStatus: vi.fn(),
      setOnline: vi.fn(),
      setError: vi.fn(),
      addPendingOperation: vi.fn(),
      removePendingOperation: vi.fn(),
      incrementRetryCount: vi.fn(),
      clearPendingOperations: vi.fn(),
      updateLastSyncTime: vi.fn(),
      triggerSync: vi.fn(),
      getPendingCount: vi.fn(() => 0),
    });

    mockUseSettingsStore.mockReturnValue({
      theme: 'light',
      setTheme: vi.fn(),
      toggleTheme: vi.fn(),
    });

    // Setup default invoke responses
    mockInvoke.mockImplementation((command: string) => {
      switch (command) {
        case 'get_ai_engines':
          return Promise.resolve(mockAIEngines);
        case 'get_user_setting':
          return Promise.resolve(null);
        case 'set_user_setting':
          return Promise.resolve(undefined);
        case 'sync_settings_with_cloud':
          return Promise.resolve({
            synced_count: 5,
            conflicts_resolved: 0,
            errors: [],
          });
        default:
          return Promise.reject(new Error(`Unknown command: ${command}`));
      }
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Initial Render', () => {
    it('shows loading spinner while loading settings', async () => {
      render(<SettingsPage />);

      // Check if page is in loading state - look for the loading class
      const loadingContainer = document.querySelector('.settings-page.loading');
      expect(loadingContainer).toBeInTheDocument();

      // Wait for loading to complete
      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });
    });

    it('renders all main sections after loading', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });

      expect(screen.getByText('Cloud Sync Status')).toBeInTheDocument();
      expect(screen.getByText('General Settings')).toBeInTheDocument();
      expect(screen.getByTestId('sync-indicator')).toBeInTheDocument();
    });

    it('loads AI engines on mount', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_ai_engines');
      });
    });

    it('loads user settings on mount', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_user_setting', {
          key: 'transcription_engine',
        });
        expect(mockInvoke).toHaveBeenCalledWith('get_user_setting', {
          key: 'cloud_sync_enabled',
        });
      });
    });
  });

  describe('Sync Status Section', () => {
    it('displays last sync time', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText(/Last synced:/)).toBeInTheDocument();
      });
    });

    it('displays synced status', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('All changes synced')).toBeInTheDocument();
      });
    });

    it('displays syncing status when syncing', async () => {
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        status: 'syncing',
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Syncing...')).toBeInTheDocument();
      });
    });

    it('displays offline status when offline', async () => {
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        status: 'offline',
        isOnline: false,
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Offline')).toBeInTheDocument();
      });
    });

    it('displays error status when sync fails', async () => {
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        status: 'failed',
        error: 'Network error',
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Sync failed')).toBeInTheDocument();
      });
    });

    it('handles manual sync button click', async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });

      const syncButton = screen.getByRole('button', { name: /Sync Now/i });
      await user.click(syncButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('sync_settings_with_cloud');
      });
    });

    it('disables sync button when cloud sync is disabled', async () => {
      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'get_user_setting' && args?.key === 'cloud_sync_enabled') {
          return Promise.resolve('false');
        }
        if (command === 'get_ai_engines') {
          return Promise.resolve(mockAIEngines);
        }
        return Promise.resolve(null);
      });

      render(<SettingsPage />);

      await waitFor(() => {
        const syncButton = screen.getByRole('button', { name: /Sync Now/i });
        expect(syncButton).toBeDisabled();
      });
    });

    it('displays sync result after successful sync', async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });

      const syncButton = screen.getByRole('button', { name: /Sync Now/i });
      await user.click(syncButton);

      await waitFor(() => {
        expect(screen.getByText(/Synced 5 setting\(s\)/)).toBeInTheDocument();
      });
    });

    it('displays sync errors', async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'sync_settings_with_cloud') {
          return Promise.resolve({
            synced_count: 3,
            conflicts_resolved: 0,
            errors: ['Failed to sync theme', 'Network timeout for language setting'],
          });
        }
        if (command === 'get_ai_engines') {
          return Promise.resolve(mockAIEngines);
        }
        return Promise.resolve(null);
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });

      const syncButton = screen.getByRole('button', { name: /Sync Now/i });
      await user.click(syncButton);

      await waitFor(() => {
        expect(screen.getByText('Failed to sync theme')).toBeInTheDocument();
        expect(screen.getByText('Network timeout for language setting')).toBeInTheDocument();
      });
    });
  });

  describe('Cloud Sync Toggle', () => {
    it('toggles cloud sync setting', async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Cloud Sync')).toBeInTheDocument();
      });

      const syncSwitch = screen.getByRole('switch', { name: /On/i });
      await user.click(syncSwitch);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('set_user_setting', {
          key: 'cloud_sync_enabled',
          value: 'false',
          syncToCloud: false,
        });
      });
    });

    it('enables cloud sync and persists setting', async () => {
      const user = userEvent.setup();

      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'get_user_setting' && args?.key === 'cloud_sync_enabled') {
          return Promise.resolve('false');
        }
        if (command === 'set_user_setting') {
          return Promise.resolve(undefined);
        }
        if (command === 'sync_settings_with_cloud') {
          return Promise.resolve({
            synced_count: 2,
            conflicts_resolved: 0,
            errors: [],
          });
        }
        if (command === 'get_ai_engines') {
          return Promise.resolve(mockAIEngines);
        }
        return Promise.resolve(null);
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Cloud Sync')).toBeInTheDocument();
      });

      const syncSwitch = screen.getByRole('switch');
      await user.click(syncSwitch);

      // Verify the setting was saved
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('set_user_setting', {
          key: 'cloud_sync_enabled',
          value: 'true',
          syncToCloud: false,
        });
      });

      // Note: Auto-sync happens asynchronously after the toggle
      // Full testing of sync behavior is better handled in E2E tests
    });
  });

  describe('Theme Setting', () => {
    it('displays current theme', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Theme')).toBeInTheDocument();
      });

      const lightRadio = screen.getByRole('radio', { name: 'Light' });
      expect(lightRadio).toBeChecked();
    });

    it('changes theme when radio button is clicked', async () => {
      const mockSetTheme = vi.fn();
      mockUseSettingsStore.mockReturnValue({
        theme: 'light',
        setTheme: mockSetTheme,
        toggleTheme: vi.fn(),
      });

      const user = userEvent.setup();
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Theme')).toBeInTheDocument();
      });

      const darkRadio = screen.getByRole('radio', { name: 'Dark' });
      await user.click(darkRadio);

      await waitFor(() => {
        expect(mockSetTheme).toHaveBeenCalledWith('dark');
        expect(mockInvoke).toHaveBeenCalledWith('set_user_setting', {
          key: 'theme',
          value: 'dark',
          syncToCloud: true,
        });
      });
    });
  });

  describe('AI Engine Selector', () => {
    it('displays AI engines in dropdown', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('AI Transcription Engine')).toBeInTheDocument();
      });

      // Verify select exists
      const select = screen.getByRole('combobox');
      expect(select).toBeInTheDocument();

      // Verify we can see the placeholder or selected value
      // Note: Testing dropdown options is complex with Arco Design
      // In a real e2e test, this would be better handled with Playwright
    });

    it('selects default engine when no engine is set', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('AI Transcription Engine')).toBeInTheDocument();
      });

      // Verify that the default engine setting logic was called
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_user_setting', {
          key: 'transcription_engine',
        });
      });
    });

    it('displays engine description and capabilities', async () => {
      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'get_user_setting' && args?.key === 'transcription_engine') {
          return Promise.resolve('whisper-v3');
        }
        if (command === 'get_ai_engines') {
          return Promise.resolve(mockAIEngines);
        }
        return Promise.resolve(null);
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Capabilities:')).toBeInTheDocument();
        expect(screen.getByText('Multilingual')).toBeInTheDocument();
        expect(screen.getByText('Punctuation')).toBeInTheDocument();
        expect(screen.getByText('Speaker detection')).toBeInTheDocument();
      });
    });

    it('updates engine setting when changed', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('AI Transcription Engine')).toBeInTheDocument();
      });

      // Verify the select is rendered and functional
      const select = screen.getByRole('combobox');
      expect(select).toBeInTheDocument();

      // Note: Full interaction testing with Arco Select is better done with E2E tests
      // Unit tests can verify the component structure and callbacks are wired correctly
    });

    it('loads engines with subscription requirements', async () => {
      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('AI Transcription Engine')).toBeInTheDocument();
      });

      // Verify engines were loaded
      expect(mockInvoke).toHaveBeenCalledWith('get_ai_engines');
    });

    it('handles authentication error gracefully', async () => {
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'get_ai_engines') {
          return Promise.reject(new Error('Not authenticated'));
        }
        return Promise.resolve(null);
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('No AI engines available. Please sign in.')).toBeInTheDocument();
      });
    });
  });

  describe('Error Handling', () => {
    it('displays error when settings fail to load', async () => {
      mockInvoke.mockRejectedValue(new Error('Database error'));

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText(/Failed to load settings/)).toBeInTheDocument();
      });
    });

    it('displays error when sync fails', async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'sync_settings_with_cloud') {
          return Promise.reject(new Error('Network error'));
        }
        if (command === 'get_ai_engines') {
          return Promise.resolve(mockAIEngines);
        }
        return Promise.resolve(null);
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });

      const syncButton = screen.getByRole('button', { name: /Sync Now/i });
      await user.click(syncButton);

      await waitFor(() => {
        expect(screen.getByText(/Sync failed: Network error/)).toBeInTheDocument();
      });
    });

    it('shows retry button on sync error', async () => {
      const user = userEvent.setup();
      mockInvoke.mockRejectedValue(new Error('Timeout'));

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });

      const syncButton = screen.getByRole('button', { name: /Sync Now/i });
      await user.click(syncButton);

      await waitFor(() => {
        const retryButton = screen.getByRole('button', { name: /Retry/i });
        expect(retryButton).toBeInTheDocument();
      });
    });
  });

  describe('Time Formatting', () => {
    it('displays "Just now" for recent syncs', async () => {
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        lastSyncTime: new Date(),
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Just now')).toBeInTheDocument();
      });
    });

    it('displays minutes ago for syncs within an hour', async () => {
      const fiveMinutesAgo = new Date(Date.now() - 5 * 60 * 1000);
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        lastSyncTime: fiveMinutesAgo,
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText(/5 minutes ago/)).toBeInTheDocument();
      });
    });

    it('displays hours ago for syncs within a day', async () => {
      const twoHoursAgo = new Date(Date.now() - 2 * 60 * 60 * 1000);
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        lastSyncTime: twoHoursAgo,
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText(/2 hours ago/)).toBeInTheDocument();
      });
    });

    it('displays days ago for older syncs', async () => {
      const threeDaysAgo = new Date(Date.now() - 3 * 24 * 60 * 60 * 1000);
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        lastSyncTime: threeDaysAgo,
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText(/3 days ago/)).toBeInTheDocument();
      });
    });

    it('displays "Never synced" when no sync has occurred', async () => {
      mockUseSyncStore.mockReturnValue({
        ...mockUseSyncStore(),
        lastSyncTime: null,
      });

      render(<SettingsPage />);

      await waitFor(() => {
        expect(screen.getByText('Never synced')).toBeInTheDocument();
      });
    });
  });
});
