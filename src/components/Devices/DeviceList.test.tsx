import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { DeviceList } from './DeviceList';
import { useDevicesStore } from '../../store/devicesStore';
import type { Device } from '../../types/devices';

// Mock the store
vi.mock('../../store/devicesStore');

// Mock Arco Design icons
vi.mock('@arco-design/web-react/icon', () => ({
  IconWifi: () => <div data-testid="icon-wifi">Wifi Icon</div>,
  IconPoweroff: () => <div data-testid="icon-poweroff">Poweroff Icon</div>,
  IconSync: () => <div data-testid="icon-sync">Sync Icon</div>,
  IconDelete: () => <div data-testid="icon-delete">Delete Icon</div>,
}));

describe('DeviceList', () => {
  const mockDevices: Device[] = [
    {
      id: 'HIDOC-001',
      name: 'My HiDoc P1',
      status: 'connected',
      last_sync: new Date().toISOString(),
      created_at: new Date().toISOString(),
    },
    {
      id: 'HIDOC-002',
      name: 'Office HiDoc',
      status: 'disconnected',
      last_sync: null,
      created_at: new Date().toISOString(),
    },
  ];

  const mockFetchDevices = vi.fn();
  const mockUnbindDevice = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: [],
      loading: false,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });
  });

  it('renders loading state', () => {
    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: [],
      loading: true,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('renders error state', () => {
    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: [],
      loading: false,
      error: 'Failed to load devices',
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);
    expect(screen.getByText(/Error loading devices: Failed to load devices/i)).toBeInTheDocument();
  });

  it('renders empty state when no devices', () => {
    render(<DeviceList />);
    expect(
      screen.getByText(/No devices bound yet. Click 'Bind Device' to add your first HiDoc P1 device./i)
    ).toBeInTheDocument();
  });

  it('renders device list with devices', () => {
    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: mockDevices,
      loading: false,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);

    expect(screen.getByText('My HiDoc P1')).toBeInTheDocument();
    expect(screen.getByText('Office HiDoc')).toBeInTheDocument();
    expect(screen.getByText('ID: HIDOC-001')).toBeInTheDocument();
    expect(screen.getByText('ID: HIDOC-002')).toBeInTheDocument();
  });

  it('displays correct status badges', () => {
    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: mockDevices,
      loading: false,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);

    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });

  it('fetches devices on mount', () => {
    render(<DeviceList />);
    expect(mockFetchDevices).toHaveBeenCalledTimes(1);
  });

  it('calls unbindDevice when unbind button is clicked', async () => {
    const user = userEvent.setup();
    mockUnbindDevice.mockResolvedValue(undefined);

    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: [mockDevices[0]], // Only include one device to avoid duplicate buttons
      loading: false,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);

    const unbindButton = screen.getByText('Unbind');
    await user.click(unbindButton);

    // Wait for Popconfirm to appear
    await waitFor(() => {
      expect(screen.getByText('Unbind Device')).toBeInTheDocument();
    });

    // Click the OK button in the Popconfirm
    // Use fireEvent instead of user.click to bypass pointer-events check
    const confirmButtons = screen.getAllByRole('button', { name: /unbind/i });
    const okButton = confirmButtons.find((button) =>
      button.classList.contains('arco-btn-primary')
    );

    if (okButton) {
      fireEvent.click(okButton);
    }

    await waitFor(() => {
      expect(mockUnbindDevice).toHaveBeenCalledWith('HIDOC-001');
    });
  });

  it('formats last sync time correctly', () => {
    const now = new Date();
    const fiveMinutesAgo = new Date(now.getTime() - 5 * 60 * 1000);

    const devicesWithSync: Device[] = [
      {
        id: 'HIDOC-001',
        name: 'Test Device',
        status: 'connected',
        last_sync: fiveMinutesAgo.toISOString(),
        created_at: now.toISOString(),
      },
    ];

    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: devicesWithSync,
      loading: false,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);

    expect(screen.getByText(/5 minutes ago/i)).toBeInTheDocument();
  });

  it('shows "Never" for devices with no last sync', () => {
    const deviceWithoutSync: Device[] = [
      {
        id: 'HIDOC-001',
        name: 'Test Device',
        status: 'disconnected',
        last_sync: null,
        created_at: new Date().toISOString(),
      },
    ];

    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      devices: deviceWithoutSync,
      loading: false,
      error: null,
      fetchDevices: mockFetchDevices,
      unbindDevice: mockUnbindDevice,
    });

    render(<DeviceList />);

    expect(screen.getByText(/Never/i)).toBeInTheDocument();
  });
});
