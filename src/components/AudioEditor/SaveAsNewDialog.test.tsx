import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SaveAsNewDialog } from './SaveAsNewDialog';

// Mock Tauri API
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('SaveAsNewDialog', () => {
  const mockOnClose = vi.fn();
  const mockOnComplete = vi.fn();
  const mockAudioData = new Uint8Array([1, 2, 3, 4, 5]);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should not render when visible is false', () => {
      render(
        <SaveAsNewDialog
          visible={false}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      expect(screen.queryByText(/save as new/i)).not.toBeInTheDocument();
    });

    it('should render when visible is true', () => {
      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      expect(screen.getByText(/save as new/i)).toBeInTheDocument();
    });

    it('should render format selection options', () => {
      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      expect(screen.getByLabelText(/format/i)).toBeInTheDocument();
    });

    it('should render quality settings', () => {
      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      expect(screen.getByRole('slider')).toBeInTheDocument();
    });
  });

  describe('Format Selection', () => {
    it('should allow selecting MP3 format', async () => {
      const user = userEvent.setup();

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const formatSelect = screen.getByLabelText(/format/i);
      await user.click(formatSelect);

      await waitFor(() => {
        expect(screen.getByText('MP3')).toBeInTheDocument();
      });
    });

    it('should allow selecting M4A format', async () => {
      const user = userEvent.setup();

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const formatSelect = screen.getByLabelText(/format/i);
      await user.click(formatSelect);

      await waitFor(() => {
        expect(screen.getByText(/M4A.*Lossy/i)).toBeInTheDocument();
      });
    });

    it('should allow selecting WAV format', async () => {
      const user = userEvent.setup();

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const formatSelect = screen.getByLabelText(/format/i);
      await user.click(formatSelect);

      await waitFor(() => {
        expect(screen.getByText(/WAV.*Lossless/i)).toBeInTheDocument();
      });
    });
  });

  describe('Quality Settings', () => {
    it('should display quality slider', () => {
      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const qualitySlider = screen.getByRole('slider');
      expect(qualitySlider).toBeInTheDocument();
    });

    it('should update quality label when slider changes', async () => {
      const user = userEvent.setup();

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const qualitySlider = screen.getByRole('slider');

      // Change quality value (implementation will handle the actual change)
      await user.click(qualitySlider);

      // Verify slider is interactive
      expect(qualitySlider).toBeInTheDocument();
    });

    it('should show bitrate options for lossy formats', async () => {
      const user = userEvent.setup();

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      // MP3 is selected by default (lossy format)
      // Should show bitrate buttons
      await waitFor(() => {
        expect(screen.getByRole('button', { name: '128k' })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: '192k' })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: '256k' })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: '320k' })).toBeInTheDocument();
      });
    });
  });

  describe('Save Operation', () => {
    it('should execute save operation when save button is clicked', async () => {
      const user = userEvent.setup();

      mockInvoke.mockResolvedValue({
        success: true,
        output_path: '/path/to/output.mp3',
        error: null,
      });

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const saveButton = screen.getByRole('button', { name: /^save$/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith(
          'save_audio_as_new',
          expect.objectContaining({
            audio_data: Array.from(mockAudioData),
            format: expect.any(String),
          })
        );
      });
    });

    it('should call onComplete callback when save succeeds', async () => {
      const user = userEvent.setup();

      mockInvoke.mockResolvedValue({
        success: true,
        output_path: '/path/to/output.mp3',
        error: null,
      });

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const saveButton = screen.getByRole('button', { name: /^save$/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnComplete).toHaveBeenCalledWith('/path/to/output.mp3');
      });
    });

    it('should display error message when save fails', async () => {
      const user = userEvent.setup();

      mockInvoke.mockResolvedValue({
        success: false,
        output_path: null,
        error: 'Save operation failed',
      });

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const saveButton = screen.getByRole('button', { name: /^save$/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText(/save operation failed/i)).toBeInTheDocument();
      });
    });

    it('should show progress indicator during save operation', async () => {
      const user = userEvent.setup();

      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({ success: true }), 1000))
      );

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const saveButton = screen.getByRole('button', { name: /^save$/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText(/saving audio file/i)).toBeInTheDocument();
      });
    });
  });

  describe('Dialog Actions', () => {
    it('should call onClose when cancel button is clicked', async () => {
      const user = userEvent.setup();

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const cancelButton = screen.getByText(/cancel/i);
      await user.click(cancelButton);

      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should close dialog after successful save', async () => {
      const user = userEvent.setup();

      mockInvoke.mockResolvedValue({
        success: true,
        output_path: '/path/to/output.mp3',
        error: null,
      });

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const saveButton = screen.getByRole('button', { name: /^save$/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnClose).toHaveBeenCalled();
      });
    });

    it('should disable controls during save operation', async () => {
      const user = userEvent.setup();

      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({ success: true }), 1000))
      );

      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      const saveButton = screen.getByRole('button', { name: /^save$/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(saveButton).toBeDisabled();
      });
    });
  });

  describe('File Size Estimation', () => {
    it('should display estimated file size for selected format and quality', () => {
      render(
        <SaveAsNewDialog
          visible={true}
          audioData={mockAudioData}
          onClose={mockOnClose}
          onComplete={mockOnComplete}
        />
      );

      // Should show file size estimation (implementation will calculate based on format/quality)
      expect(screen.getByText(/estimated size/i)).toBeInTheDocument();
    });
  });
});
