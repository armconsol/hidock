import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { BindDeviceDialog } from './BindDeviceDialog';
import { useDevicesStore } from '../../store/devicesStore';

// Mock the store
vi.mock('../../store/devicesStore');

// Mock Arco Design icons
vi.mock('@arco-design/web-react/icon', () => ({
  IconPlus: () => <div>Plus Icon</div>,
}));

describe('BindDeviceDialog', () => {
  const mockBindDevice = vi.fn();
  const mockOnClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    (useDevicesStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector) => {
      const state = {
        bindDevice: mockBindDevice,
      };
      return selector(state);
    });
  });

  it('renders when visible', () => {
    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    expect(screen.getByText('Bind New Device')).toBeInTheDocument();
    expect(screen.getByLabelText('Device ID')).toBeInTheDocument();
    expect(screen.getByLabelText('Device Name')).toBeInTheDocument();
  });

  it('does not render when not visible', () => {
    render(<BindDeviceDialog visible={false} onClose={mockOnClose} />);

    expect(screen.queryByText('Bind New Device')).not.toBeInTheDocument();
  });

  it('validates required fields', async () => {
    const user = userEvent.setup();
    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const bindButton = screen.getByRole('button', { name: /bind device/i });
    await user.click(bindButton);

    await waitFor(() => {
      expect(screen.getByText('Please enter device ID')).toBeInTheDocument();
      expect(screen.getByText('Please enter device name')).toBeInTheDocument();
    });

    expect(mockBindDevice).not.toHaveBeenCalled();
  });

  it('validates minimum length for device ID', async () => {
    const user = userEvent.setup();
    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const deviceIdInput = screen.getByLabelText('Device ID');
    await user.type(deviceIdInput, 'AB');

    const bindButton = screen.getByRole('button', { name: /bind device/i });
    await user.click(bindButton);

    await waitFor(() => {
      expect(screen.getByText('Device ID must be at least 3 characters')).toBeInTheDocument();
    });

    expect(mockBindDevice).not.toHaveBeenCalled();
  });

  it('validates minimum length for device name', async () => {
    const user = userEvent.setup();
    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const deviceIdInput = screen.getByLabelText('Device ID');
    const deviceNameInput = screen.getByLabelText('Device Name');

    await user.type(deviceIdInput, 'HIDOC-001');
    await user.type(deviceNameInput, 'A');

    const bindButton = screen.getByRole('button', { name: /bind device/i });
    await user.click(bindButton);

    await waitFor(() => {
      expect(screen.getByText('Device name must be at least 2 characters')).toBeInTheDocument();
    });

    expect(mockBindDevice).not.toHaveBeenCalled();
  });

  it('submits form with valid data', async () => {
    const user = userEvent.setup();
    mockBindDevice.mockResolvedValue(undefined);

    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const deviceIdInput = screen.getByLabelText('Device ID');
    const deviceNameInput = screen.getByLabelText('Device Name');

    await user.type(deviceIdInput, 'HIDOC-12345');
    await user.type(deviceNameInput, 'My HiDoc P1');

    const bindButton = screen.getByRole('button', { name: /bind device/i });
    await user.click(bindButton);

    await waitFor(() => {
      expect(mockBindDevice).toHaveBeenCalledWith({
        id: 'HIDOC-12345',
        name: 'My HiDoc P1',
      });
    });

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  it('handles bind error gracefully', async () => {
    const user = userEvent.setup();
    const errorMessage = 'Device already exists';
    mockBindDevice.mockRejectedValue(new Error(errorMessage));

    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const deviceIdInput = screen.getByLabelText('Device ID');
    const deviceNameInput = screen.getByLabelText('Device Name');

    await user.type(deviceIdInput, 'HIDOC-12345');
    await user.type(deviceNameInput, 'My HiDoc P1');

    const bindButton = screen.getByRole('button', { name: /bind device/i });
    await user.click(bindButton);

    await waitFor(() => {
      expect(mockBindDevice).toHaveBeenCalled();
    });

    // Should not close dialog on error
    expect(mockOnClose).not.toHaveBeenCalled();
  });

  it('resets form when closing', async () => {
    const user = userEvent.setup();
    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const deviceIdInput = screen.getByLabelText('Device ID');
    const deviceNameInput = screen.getByLabelText('Device Name');

    await user.type(deviceIdInput, 'HIDOC-12345');
    await user.type(deviceNameInput, 'My HiDoc P1');

    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    await user.click(cancelButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it('shows loading state while binding', async () => {
    const user = userEvent.setup();
    mockBindDevice.mockImplementation(
      () => new Promise((resolve) => setTimeout(resolve, 100))
    );

    render(<BindDeviceDialog visible={true} onClose={mockOnClose} />);

    const deviceIdInput = screen.getByLabelText('Device ID');
    const deviceNameInput = screen.getByLabelText('Device Name');

    await user.type(deviceIdInput, 'HIDOC-12345');
    await user.type(deviceNameInput, 'My HiDoc P1');

    const bindButton = screen.getByRole('button', { name: /bind device/i });
    await user.click(bindButton);

    // Should show loading class
    await waitFor(() => {
      expect(bindButton.classList.contains('arco-btn-loading')).toBe(true);
    });
  });
});
