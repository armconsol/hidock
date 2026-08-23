import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { BindDeviceDialog } from './BindDeviceDialog';
import { useDevicesStore } from '../../store/devicesStore';
import { message } from 'antd';

// Mock the store
vi.mock('../../store/devicesStore');

// Mock Ant Design icons
vi.mock('@ant-design/icons', () => ({
  PlusOutlined: () => <div>Plus Icon</div>,
}));

/**
 * React 19 Compatibility Note:
 *
 * These tests use React 18+ patterns with @testing-library/react, which
 * internally uses createRoot() instead of the deprecated ReactDOM.render().
 *
 * The Ant Design message API is mocked in src/test/setup.ts to provide
 * test-compatible implementations that avoid DOM rendering errors.
 */

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

    // Verify success message is displayed via mocked message API
    await waitFor(() => {
      expect(message.success).toHaveBeenCalledWith('Device bound successfully');
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

    // Verify error message is displayed via mocked message API
    // (message.error is mocked in setup.ts for React 19 compatibility)
    await waitFor(() => {
      expect(message.error).toHaveBeenCalledWith(
        expect.stringContaining(errorMessage)
      );
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
      expect(bindButton.classList.contains('ant-btn-loading')).toBe(true);
    });
  });
});
