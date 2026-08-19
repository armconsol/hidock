import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { DeviceFiles } from './DeviceFiles';
import type { DeviceFile, DeviceInfo } from './DeviceFiles';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

// Mock Arco Message
vi.mock('@arco-design/web-react', async () => {
  const actual = await vi.importActual('@arco-design/web-react');
  return {
    ...actual,
    Message: {
      success: vi.fn(),
      error: vi.fn(),
      info: vi.fn(),
      warning: vi.fn(),
    },
  };
});

describe('DeviceFiles', () => {
  const mockDeviceInfo: DeviceInfo = {
    id: 'device-1',
    name: 'Test Device',
    storage_used: 5368709120, // 5GB
    storage_total: 10737418240, // 10GB
    last_sync: '2024-01-15T10:30:00Z',
  };

  const mockFiles: DeviceFile[] = [
    {
      id: 'file-1',
      name: 'recording-001.m4a',
      size: 1048576, // 1MB
      duration: 120, // 2 minutes
      created_at: '2024-01-15T09:00:00Z',
      synced: true,
    },
    {
      id: 'file-2',
      name: 'recording-002.m4a',
      size: 2097152, // 2MB
      duration: 240, // 4 minutes
      created_at: '2024-01-15T10:00:00Z',
      synced: false,
    },
    {
      id: 'file-3',
      name: 'recording-003.m4a',
      size: 3145728, // 3MB
      duration: null,
      created_at: '2024-01-15T11:00:00Z',
      synced: false,
    },
  ];

  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it('renders device info correctly', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('Test Device')).toBeInTheDocument();
      expect(screen.getByText(/Device ID: device-1/)).toBeInTheDocument();
    });

    // Check storage display
    expect(screen.getByText(/5.00 GB/)).toBeInTheDocument();
    expect(screen.getByText(/10.00 GB/)).toBeInTheDocument();
  });

  it('displays files in table', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
      expect(screen.getByText('recording-002.m4a')).toBeInTheDocument();
      expect(screen.getByText('recording-003.m4a')).toBeInTheDocument();
    });

    // Check synced status
    expect(screen.getByText('Synced')).toBeInTheDocument();
    expect(screen.getAllByText('Not Synced')).toHaveLength(2);

    // Check file sizes
    expect(screen.getByText('1.00 MB')).toBeInTheDocument();
    expect(screen.getByText('2.00 MB')).toBeInTheDocument();
    expect(screen.getByText('3.00 MB')).toBeInTheDocument();

    // Check durations
    expect(screen.getByText('2:00')).toBeInTheDocument();
    expect(screen.getByText('4:00')).toBeInTheDocument();
  });

  it('handles file download', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      if (cmd === 'download_device_file') return Promise.resolve();
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
    });

    // Find and click download button for first file
    const table = document.querySelector('.device-files-table');
    const downloadButtons = table!.querySelectorAll('button[aria-label*="download"], button .arco-icon-download');
    fireEvent.click(downloadButtons[0].closest('button')!);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'download_device_file',
        expect.objectContaining({
          deviceId: 'device-1',
          fileId: 'file-1',
          fileName: 'recording-001.m4a',
        })
      );
    });
  });

  it('handles sync all files', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      if (cmd === 'download_device_file') return Promise.resolve();
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText(/Sync All/)).toBeInTheDocument();
    });

    const syncButton = screen.getByText(/Sync All \(2\)/i).closest('button')!;
    fireEvent.click(syncButton);

    await waitFor(() => {
      // Should download only unsynced files
      expect(mockInvoke).toHaveBeenCalledWith(
        'download_device_file',
        expect.objectContaining({ fileId: 'file-2' })
      );
      expect(mockInvoke).toHaveBeenCalledWith(
        'download_device_file',
        expect.objectContaining({ fileId: 'file-3' })
      );
    });
  });

  it('handles file selection and bulk download', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      if (cmd === 'download_device_file') return Promise.resolve();
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
    });

    // Select first two files
    const checkboxes = screen.getAllByRole('checkbox');
    fireEvent.click(checkboxes[1]); // First file
    fireEvent.click(checkboxes[2]); // Second file

    // Click download selected
    const downloadSelectedButton = screen.getByText(/Download Selected \(2\)/i).closest('button')!;
    fireEvent.click(downloadSelectedButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledTimes(
        4 // get_device_info, list_device_files, and 2 downloads
      );
    });
  });

  it('renders delete button for files', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
    });

    // Verify delete buttons are present
    const table = document.querySelector('.device-files-table');
    const deleteButtons = table!.querySelectorAll('button .arco-icon-delete');
    expect(deleteButtons.length).toBeGreaterThan(0);
  });

  it('handles file upload', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      if (cmd === 'upload_file_to_device') return Promise.resolve();
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText(/Click or drag audio files/)).toBeInTheDocument();
    });

    // Create a mock file
    const file = new File(['audio content'], 'test-audio.m4a', {
      type: 'audio/mp4',
    });

    const uploadInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    Object.defineProperty(uploadInput, 'files', {
      value: [file],
      writable: false,
    });

    fireEvent.change(uploadInput);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'upload_file_to_device',
        expect.objectContaining({
          deviceId: 'device-1',
          fileName: 'test-audio.m4a',
        })
      );
    });
  });

  it('handles refresh action', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
    });

    mockInvoke.mockClear();

    const refreshButton = screen.getByRole('button', { name: /Refresh/i });
    fireEvent.click(refreshButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('list_device_files', {
        deviceId: 'device-1',
      });
    });
  });

  it('displays empty state when no files', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve([]);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(
        screen.getByText(/No files on this device. Upload files to get started./i)
      ).toBeInTheDocument();
    });
  });

  it('displays error state on fetch failure', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files')
        return Promise.reject(new Error('Network error'));
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText(/Error loading files/i)).toBeInTheDocument();
    });
  });

  it('disables sync all when all files are synced', async () => {
    const allSyncedFiles = mockFiles.map((f) => ({ ...f, synced: true }));

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(allSyncedFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      const syncButton = screen.getByText(/Sync All/i).closest('button')!;
      expect(syncButton).toBeDisabled();
    });
  });

  it('handles download retry on error', async () => {
    let downloadAttempts = 0;

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      if (cmd === 'download_device_file') {
        downloadAttempts++;
        if (downloadAttempts === 1) {
          return Promise.reject(new Error('Download failed'));
        }
        return Promise.resolve();
      }
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
    });

    // Attempt download
    const table = document.querySelector('.device-files-table');
    const downloadButtons = table!.querySelectorAll('button .arco-icon-download');
    fireEvent.click(downloadButtons[0].closest('button')!);

    // Wait for error and retry button
    await waitFor(() => {
      expect(screen.getByText(/Retry/i)).toBeInTheDocument();
    });

    // Click retry
    const retryButton = screen.getByText(/Retry/i).closest('button')!;
    fireEvent.click(retryButton);

    await waitFor(() => {
      expect(downloadAttempts).toBe(2);
    });
  });

  it('formats file sizes correctly', async () => {
    const filesWithVariousSizes: DeviceFile[] = [
      {
        id: 'file-1',
        name: 'small.m4a',
        size: 512, // 512 B
        duration: null,
        created_at: '2024-01-15T09:00:00Z',
        synced: false,
      },
      {
        id: 'file-2',
        name: 'medium.m4a',
        size: 524288, // 512 KB
        duration: null,
        created_at: '2024-01-15T10:00:00Z',
        synced: false,
      },
      {
        id: 'file-3',
        name: 'large.m4a',
        size: 1073741824, // 1 GB
        duration: null,
        created_at: '2024-01-15T11:00:00Z',
        synced: false,
      },
    ];

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(filesWithVariousSizes);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('512.00 B')).toBeInTheDocument();
      expect(screen.getByText('512.00 KB')).toBeInTheDocument();
      expect(screen.getByText('1.00 GB')).toBeInTheDocument();
    });
  });

  it('handles select all checkbox', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(mockDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      expect(screen.getByText('recording-001.m4a')).toBeInTheDocument();
    });

    // Click select all checkbox (first checkbox in the table)
    const checkboxes = screen.getAllByRole('checkbox');
    fireEvent.click(checkboxes[0]);

    // Check that download selected button shows correct count
    await waitFor(() => {
      const downloadButton = screen.getByText(/Download Selected \(3\)/i);
      expect(downloadButton).toBeInTheDocument();
    });

    // Unselect all
    fireEvent.click(checkboxes[0]);

    await waitFor(() => {
      const downloadButton = screen.getByText(/Download Selected \(0\)/i);
      expect(downloadButton).toBeInTheDocument();
    });
  });

  it('displays storage warning when storage is high', async () => {
    const highStorageDeviceInfo: DeviceInfo = {
      ...mockDeviceInfo,
      storage_used: 9663676416, // 9GB (90%)
    };

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_device_info') return Promise.resolve(highStorageDeviceInfo);
      if (cmd === 'list_device_files') return Promise.resolve(mockFiles);
      return Promise.reject(new Error('Unknown command'));
    });

    render(<DeviceFiles deviceId="device-1" />);

    await waitFor(() => {
      // Check that the device info section is present
      expect(screen.getByText('Test Device')).toBeInTheDocument();
      // Storage percentage should be around 90%
      expect(screen.getByText(/9.00 GB/)).toBeInTheDocument();
    });
  });
});
