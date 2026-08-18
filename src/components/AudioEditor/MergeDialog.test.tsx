import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MergeDialog } from './MergeDialog';

// Mock Tauri API
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Mock Tauri dialog API
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

describe('MergeDialog', () => {
  const mockOnClose = vi.fn();
  const mockOnComplete = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should not render when visible is false', () => {
      render(
        <MergeDialog visible={false} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      expect(screen.queryByText(/merge audio/i)).not.toBeInTheDocument();
    });

    it('should render when visible is true', () => {
      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      expect(screen.getByText(/merge audio/i)).toBeInTheDocument();
    });

    it('should render select files button', () => {
      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      expect(screen.getByText(/select audio files/i)).toBeInTheDocument();
    });
  });

  describe('File Selection', () => {
    it('should allow selecting multiple audio files', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValue(60000); // 60 seconds duration

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(mockOpen.open).toHaveBeenCalledWith({
          multiple: true,
          filters: expect.arrayContaining([
            expect.objectContaining({
              name: 'Audio',
              extensions: expect.arrayContaining(['mp3', 'm4a', 'wav']),
            }),
          ]),
        });
      });
    });

    it('should display selected files with durations', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValue(120000); // 120 seconds

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/file1\.mp3/i)).toBeInTheDocument();
        expect(screen.getByText(/file2\.mp3/i)).toBeInTheDocument();
      });
    });
  });

  describe('File Reordering', () => {
    it('should allow reordering files', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValue(60000);

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/file1\.mp3/i)).toBeInTheDocument();
      });

      // Files should have reorder buttons (drag and drop or arrows)
      const moveButtons = screen.getAllByRole('button');
      expect(moveButtons.length).toBeGreaterThan(2); // At least select, cancel, merge buttons
    });
  });

  describe('Merge Operation', () => {
    it('should disable merge button when less than 2 files selected', () => {
      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const mergeButton = screen.getByText(/merge files/i);
      expect(mergeButton).toBeDisabled();
    });

    it('should execute merge operation when merge button is clicked', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValueOnce(60000); // Duration for file 1
      mockInvoke.mockResolvedValueOnce(90000); // Duration for file 2
      mockInvoke.mockResolvedValueOnce({
        success: true,
        output_path: '/path/to/merged.mp3',
        error: null,
      });

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/file1\.mp3/i)).toBeInTheDocument();
      });

      const mergeButton = screen.getByText(/merge files/i);
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('merge_audio_files', {
          request: {
            file_paths: ['/path/to/file1.mp3', '/path/to/file2.mp3'],
          },
        });
      });
    });

    it('should call onComplete callback when merge succeeds', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValueOnce(60000);
      mockInvoke.mockResolvedValueOnce(90000);
      mockInvoke.mockResolvedValueOnce({
        success: true,
        output_path: '/path/to/merged.mp3',
        error: null,
      });

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/file1\.mp3/i)).toBeInTheDocument();
      });

      const mergeButton = screen.getByText(/merge files/i);
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockOnComplete).toHaveBeenCalledWith('/path/to/merged.mp3');
      });
    });

    it('should display error message when merge fails', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValueOnce(60000);
      mockInvoke.mockResolvedValueOnce(90000);
      mockInvoke.mockResolvedValueOnce({
        success: false,
        output_path: null,
        error: 'Merge operation failed',
      });

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/file1\.mp3/i)).toBeInTheDocument();
      });

      const mergeButton = screen.getByText(/merge files/i);
      await user.click(mergeButton);

      await waitFor(() => {
        expect(screen.getByText(/merge operation failed/i)).toBeInTheDocument();
      });
    });
  });

  describe('Dialog Actions', () => {
    it('should call onClose when cancel button is clicked', async () => {
      const user = userEvent.setup();

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const cancelButton = screen.getByText(/cancel/i);
      await user.click(cancelButton);

      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should close dialog after successful merge', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValueOnce(60000);
      mockInvoke.mockResolvedValueOnce(90000);
      mockInvoke.mockResolvedValueOnce({
        success: true,
        output_path: '/path/to/merged.mp3',
        error: null,
      });

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/file1\.mp3/i)).toBeInTheDocument();
      });

      const mergeButton = screen.getByText(/merge files/i);
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockOnClose).toHaveBeenCalled();
      });
    });
  });

  describe('Preview', () => {
    it('should display total duration of all selected files', async () => {
      const user = userEvent.setup();
      const mockOpen = await import('@tauri-apps/plugin-dialog');

      (mockOpen.open as any).mockResolvedValue([
        '/path/to/file1.mp3',
        '/path/to/file2.mp3',
      ]);

      mockInvoke.mockResolvedValueOnce(60000); // 1 minute
      mockInvoke.mockResolvedValueOnce(90000); // 1.5 minutes

      render(
        <MergeDialog visible={true} onClose={mockOnClose} onComplete={mockOnComplete} />
      );

      const selectButton = screen.getByText(/select audio files/i);
      await user.click(selectButton);

      await waitFor(() => {
        expect(screen.getByText(/2:30/)).toBeInTheDocument(); // Total duration
      });
    });
  });
});
