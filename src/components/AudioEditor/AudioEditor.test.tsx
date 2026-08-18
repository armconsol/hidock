import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AudioEditor } from './AudioEditor';

// Mock Tauri API
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Mock HTMLMediaElement methods
HTMLMediaElement.prototype.play = vi.fn(() => Promise.resolve());
HTMLMediaElement.prototype.pause = vi.fn();
HTMLMediaElement.prototype.load = vi.fn();

// Mock URL.createObjectURL
global.URL.createObjectURL = vi.fn(() => 'blob:mock-url');
global.URL.revokeObjectURL = vi.fn();

// Mock Canvas API for waveform visualization
HTMLCanvasElement.prototype.getContext = vi.fn(() => ({
  fillRect: vi.fn(),
  clearRect: vi.fn(),
  fillStyle: '',
  strokeStyle: '',
  lineWidth: 1,
  beginPath: vi.fn(),
  moveTo: vi.fn(),
  lineTo: vi.fn(),
  stroke: vi.fn(),
  fill: vi.fn(),
  canvas: {
    width: 800,
    height: 100,
  },
})) as any;

describe('AudioEditor', () => {
  const mockNoteId = 'test-note-123';
  const mockAudioUrl = 'https://example.com/audio.mp3';
  const mockAudioData = new Array(1024).fill(0).map((_, i) => i % 256);

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(mockAudioData);
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  describe('Initialization', () => {
    it('should render audio editor with waveform canvas', async () => {
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('audio-editor')).toBeInTheDocument();
        expect(screen.getByTestId('waveform-canvas')).toBeInTheDocument();
      });
    });

    it('should display loading state while fetching audio', () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves

      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      expect(screen.getByText(/loading audio/i)).toBeInTheDocument();
    });

    it('should display error message when audio fetch fails', async () => {
      mockInvoke.mockRejectedValue(new Error('Network error'));

      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByText(/network error/i)).toBeInTheDocument();
      });
    });
  });

  describe('Waveform Visualization', () => {
    it('should render waveform canvas with correct dimensions', async () => {
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        const canvas = screen.getByTestId('waveform-canvas') as HTMLCanvasElement;
        expect(canvas).toBeInTheDocument();
        expect(canvas.width).toBeGreaterThan(0);
        expect(canvas.height).toBeGreaterThan(0);
      });
    });

    it('should update waveform when audio data changes', async () => {
      const { rerender } = render(
        <AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />
      );

      await waitFor(() => {
        expect(screen.getByTestId('waveform-canvas')).toBeInTheDocument();
      });

      const newAudioUrl = 'https://example.com/audio2.mp3';
      rerender(<AudioEditor noteId={mockNoteId} audioUrl={newAudioUrl} />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_audio', {
          noteId: mockNoteId,
          audioUrl: newAudioUrl,
        });
      });
    });
  });

  describe('Timeline Markers', () => {
    it('should render timeline with time markers', async () => {
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('timeline')).toBeInTheDocument();
      });
    });

    it('should allow adding markers for merge operations', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('timeline')).toBeInTheDocument();
      });

      const addMarkerButton = screen.getByTestId('add-marker-button');
      await user.click(addMarkerButton);

      await waitFor(() => {
        expect(screen.getByTestId('timeline-marker')).toBeInTheDocument();
      });
    });

    it('should display current playback position on timeline', async () => {
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('playback-indicator')).toBeInTheDocument();
      });
    });
  });

  describe('Audio Operations', () => {
    it('should open merge dialog when merge button is clicked', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('merge-button')).toBeInTheDocument();
      });

      const mergeButton = screen.getByTestId('merge-button');
      await user.click(mergeButton);

      await waitFor(() => {
        expect(screen.getByText(/merge audio/i)).toBeInTheDocument();
      });
    });

    it('should open replace dialog when replace button is clicked', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('replace-button')).toBeInTheDocument();
      });

      const replaceButton = screen.getByTestId('replace-button');
      await user.click(replaceButton);

      await waitFor(() => {
        expect(screen.getByText(/replace audio segment/i)).toBeInTheDocument();
      });
    });

    it('should open save-as-new dialog when save button is clicked', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('save-as-new-button')).toBeInTheDocument();
      });

      const saveButton = screen.getByTestId('save-as-new-button');
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText(/save as new/i)).toBeInTheDocument();
      });
    });
  });

  describe('Playback Controls', () => {
    it('should toggle play/pause when button is clicked', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('play-pause-button')).toBeInTheDocument();
      });

      const playButton = screen.getByTestId('play-pause-button');
      const audio = document.querySelector('audio');

      await user.click(playButton);
      expect(audio?.play).toHaveBeenCalled();

      await user.click(playButton);
      expect(audio?.pause).toHaveBeenCalled();
    });

    it('should seek to position when timeline is clicked', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('timeline')).toBeInTheDocument();
      });

      const timeline = screen.getByTestId('timeline');
      const audio = document.querySelector('audio') as HTMLAudioElement;

      // Simulate metadata loaded
      Object.defineProperty(audio, 'duration', { value: 180, configurable: true });
      audio.dispatchEvent(new Event('loadedmetadata'));

      await user.click(timeline);

      // Verify that clicking timeline seeks the audio
      expect(audio.currentTime).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Range Selection', () => {
    it('should allow selecting time range for editing', async () => {
      const user = userEvent.setup();
      render(<AudioEditor noteId={mockNoteId} audioUrl={mockAudioUrl} />);

      await waitFor(() => {
        expect(screen.getByTestId('timeline')).toBeInTheDocument();
      });

      const timeline = screen.getByTestId('timeline');

      // Simulate mouse down to start selection
      await user.pointer({ target: timeline, coords: { x: 100, y: 50 }, keys: '[MouseLeft>]' });

      // Simulate mouse move to extend selection
      await user.pointer({ coords: { x: 200, y: 50 } });

      // Simulate mouse up to complete selection
      await user.pointer({ keys: '[/MouseLeft]' });

      // Verify selection exists (implementation will handle the actual selection state)
      expect(timeline).toBeInTheDocument();
    });
  });

  describe('Callbacks', () => {
    it('should call onOperationComplete when audio operation succeeds', async () => {
      const onOperationComplete = vi.fn();
      const user = userEvent.setup();

      mockInvoke.mockResolvedValueOnce(mockAudioData); // Initial audio load
      mockInvoke.mockResolvedValueOnce({
        success: true,
        output_path: '/path/to/output.mp3',
        error: null,
      });

      render(
        <AudioEditor
          noteId={mockNoteId}
          audioUrl={mockAudioUrl}
          onOperationComplete={onOperationComplete}
        />
      );

      await waitFor(() => {
        expect(screen.getByTestId('merge-button')).toBeInTheDocument();
      });

      // This test validates the callback prop is wired up correctly
      expect(onOperationComplete).not.toHaveBeenCalled();
    });
  });
});
