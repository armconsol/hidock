import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { WhisperRecorder } from './WhisperRecorder';

// Mock Ant Design icons
vi.mock('@ant-design/icons', () => ({
  StopOutlined: () => <span>Stop Icon</span>,
  AudioOutlined: () => <span>Voice Icon</span>,
}));

// Mock MediaRecorder
class MockMediaRecorder {
  state: 'inactive' | 'recording' | 'paused' = 'inactive';
  ondataavailable: ((event: { data: Blob }) => void) | null = null;
  onstop: (() => void) | null = null;

  constructor(public stream: MediaStream, public options?: MediaRecorderOptions) {}

  start() {
    this.state = 'recording';
  }

  stop() {
    this.state = 'inactive';
    if (this.onstop) {
      this.onstop();
    }
  }
}

// Mock MediaStream
class MockMediaStream {
  getTracks() {
    return [
      {
        stop: vi.fn(),
      },
    ];
  }
}

describe('WhisperRecorder Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Mock getUserMedia
    Object.defineProperty(navigator, 'mediaDevices', {
      writable: true,
      value: {
        getUserMedia: vi.fn().mockResolvedValue(new MockMediaStream()),
      },
    });

    // Mock MediaRecorder
    (window as any).MediaRecorder = MockMediaRecorder;
  });

  it('renders with default state', () => {
    render(<WhisperRecorder />);

    expect(screen.getByTestId('whisper-recorder')).toBeInTheDocument();
    expect(screen.getByTestId('record-button')).toBeInTheDocument();
    expect(screen.getByText('Press and hold to record')).toBeInTheDocument();
  });

  it('displays max duration instruction', () => {
    render(<WhisperRecorder maxDuration={60} />);

    expect(screen.getByText(/Max duration: 1:00/)).toBeInTheDocument();
  });

  it('displays default max duration', () => {
    render(<WhisperRecorder />);

    expect(screen.getByText(/Max duration: 2:00/)).toBeInTheDocument();
  });

  it('renders voice icon when not recording', () => {
    render(<WhisperRecorder />);

    expect(screen.getByText('Voice Icon')).toBeInTheDocument();
  });

  it('renders record button with circle shape', () => {
    render(<WhisperRecorder />);

    const recordButton = screen.getByTestId('record-button');
    expect(recordButton).toHaveClass('ant-btn-circle');
  });

  it('displays correct instruction text', () => {
    render(<WhisperRecorder />);

    expect(screen.getByText('Press and hold to record')).toBeInTheDocument();
  });

  it('formats duration correctly for 60 seconds', () => {
    render(<WhisperRecorder maxDuration={60} />);

    expect(screen.getByText(/1:00/)).toBeInTheDocument();
  });

  it('formats duration correctly for 120 seconds', () => {
    render(<WhisperRecorder maxDuration={120} />);

    expect(screen.getByText(/2:00/)).toBeInTheDocument();
  });

  it('formats duration correctly for 180 seconds', () => {
    render(<WhisperRecorder maxDuration={180} />);

    expect(screen.getByText(/3:00/)).toBeInTheDocument();
  });

  it('handles microphone access error display', async () => {
    // Mock getUserMedia to reject
    navigator.mediaDevices.getUserMedia = vi.fn().mockRejectedValue(
      new Error('Permission denied')
    );

    render(<WhisperRecorder />);

    // Component should render without errors even if permission will be denied
    expect(screen.getByTestId('whisper-recorder')).toBeInTheDocument();
  });

  it('accepts onRecordingComplete callback', () => {
    const callback = vi.fn();
    render(<WhisperRecorder onRecordingComplete={callback} />);

    expect(screen.getByTestId('whisper-recorder')).toBeInTheDocument();
  });

  it('accepts custom maxDuration', () => {
    render(<WhisperRecorder maxDuration={300} />);

    expect(screen.getByText(/Max duration: 5:00/)).toBeInTheDocument();
  });

  it('renders with primary button type', () => {
    render(<WhisperRecorder />);

    const recordButton = screen.getByTestId('record-button');
    expect(recordButton).toHaveClass('ant-btn-primary');
  });

  it('renders with large button size', () => {
    render(<WhisperRecorder />);

    const recordButton = screen.getByTestId('record-button');
    expect(recordButton).toHaveClass('ant-btn-lg');
  });
});
