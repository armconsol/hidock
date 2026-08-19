import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { WhisperList } from './WhisperList';
import type { Whisper } from '../../types/whispers';

// Mock Arco Design icons
vi.mock('@arco-design/web-react/icon', () => ({
  IconClockCircle: () => <span>Clock Icon</span>,
  IconDelete: () => <span>Delete Icon</span>,
  IconFile: () => <span>File Icon</span>,
  IconCheckCircle: () => <span>Check Icon</span>,
  IconCalendar: () => <span>Calendar Icon</span>,
}));

describe('WhisperList Component', () => {
  const mockWhispers: Whisper[] = [
    {
      id: '1',
      content: 'Test whisper content 1',
      audioUrl: '/test-audio-1.webm',
      duration: 30,
      createdAt: new Date('2026-08-19T10:00:00Z'),
      updatedAt: new Date('2026-08-19T10:00:00Z'),
    },
    {
      id: '2',
      content: 'Test whisper content 2 with a much longer text that should be truncated when displayed in the list view',
      audioUrl: '/test-audio-2.webm',
      duration: 120,
      createdAt: new Date('2026-08-18T15:30:00Z'),
      updatedAt: new Date('2026-08-18T15:30:00Z'),
    },
  ];

  const mockOnDelete = vi.fn();
  const mockOnConvert = vi.fn();
  const mockOnPlayAudio = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders loading state', () => {
    render(<WhisperList whispers={[]} loading={true} />);

    expect(screen.getByTestId('whisper-list-loading')).toBeInTheDocument();
    expect(screen.getByText('Loading whispers...')).toBeInTheDocument();
  });

  it('renders empty state when no whispers', () => {
    render(<WhisperList whispers={[]} />);

    expect(screen.getByTestId('whisper-list-empty')).toBeInTheDocument();
    expect(screen.getByText('No whispers yet')).toBeInTheDocument();
  });

  it('renders list of whispers', () => {
    render(<WhisperList whispers={mockWhispers} />);

    expect(screen.getByTestId('whisper-list')).toBeInTheDocument();
    expect(screen.getByTestId('whisper-card-0')).toBeInTheDocument();
    expect(screen.getByTestId('whisper-card-1')).toBeInTheDocument();
  });

  it('displays whisper content', () => {
    render(<WhisperList whispers={mockWhispers} />);

    expect(screen.getByText('Test whisper content 1')).toBeInTheDocument();
    expect(screen.getByText(/Test whisper content 2 with a much longer text/)).toBeInTheDocument();
  });

  it('displays duration in correct format', () => {
    render(<WhisperList whispers={mockWhispers} />);

    // 30 seconds = 0:30
    expect(screen.getByText(/0:30/)).toBeInTheDocument();
    // 120 seconds = 2:00
    expect(screen.getByText(/2:00/)).toBeInTheDocument();
  });

  it('displays relative timestamps', () => {
    render(<WhisperList whispers={mockWhispers} />);

    // Should show relative time like "2h ago", "Yesterday", etc.
    expect(screen.getAllByText(/ago|Yesterday|Today/i).length).toBeGreaterThan(0);
  });

  it('calls onDelete when delete button is clicked', async () => {
    const user = userEvent.setup();
    render(
      <WhisperList
        whispers={mockWhispers}
        onDelete={mockOnDelete}
      />
    );

    const deleteButton = screen.getByTestId('delete-button-0');
    await user.click(deleteButton);

    expect(mockOnDelete).toHaveBeenCalledWith('1');
  });

  it('calls onPlayAudio when play button is clicked', async () => {
    const user = userEvent.setup();
    render(
      <WhisperList
        whispers={mockWhispers}
        onPlayAudio={mockOnPlayAudio}
      />
    );

    const playButton = screen.getByTestId('play-button-0');
    await user.click(playButton);

    expect(mockOnPlayAudio).toHaveBeenCalledWith('/test-audio-1.webm');
  });

  it('calls onConvert with correct target type', async () => {
    const user = userEvent.setup();
    render(
      <WhisperList
        whispers={mockWhispers}
        onConvert={mockOnConvert}
      />
    );

    // Get convert buttons for first whisper
    const cards = screen.getAllByTestId(/^whisper-card-/);
    expect(cards.length).toBe(2);

    // Click convert to note button
    const noteButtons = screen.getAllByTestId('convert-to-note');
    await user.click(noteButtons[0]);
    expect(mockOnConvert).toHaveBeenCalledWith('1', 'note');

    // Click convert to todo button
    const todoButtons = screen.getAllByTestId('convert-to-todo');
    await user.click(todoButtons[0]);
    expect(mockOnConvert).toHaveBeenCalledWith('1', 'todo');

    // Click convert to calendar button
    const calendarButtons = screen.getAllByTestId('convert-to-calendar');
    await user.click(calendarButtons[0]);
    expect(mockOnConvert).toHaveBeenCalledWith('1', 'calendar');
  });

  it('renders all action buttons for each whisper', () => {
    render(<WhisperList whispers={mockWhispers} />);

    expect(screen.getAllByTestId('convert-to-note')).toHaveLength(2);
    expect(screen.getAllByTestId('convert-to-todo')).toHaveLength(2);
    expect(screen.getAllByTestId('convert-to-calendar')).toHaveLength(2);
  });

  it('displays clock icon for timestamps', () => {
    render(<WhisperList whispers={mockWhispers} />);

    // Icons are mocked as spans with text
    const clockIcons = screen.getAllByText('Clock Icon');
    expect(clockIcons.length).toBeGreaterThan(0);
  });

  it('formats duration correctly for various lengths', () => {
    const whispers: Whisper[] = [
      { ...mockWhispers[0], id: '1', duration: 5 },    // 0:05
      { ...mockWhispers[0], id: '2', duration: 59 },   // 0:59
      { ...mockWhispers[0], id: '3', duration: 60 },   // 1:00
      { ...mockWhispers[0], id: '4', duration: 125 },  // 2:05
    ];

    render(<WhisperList whispers={whispers} />);

    expect(screen.getByText(/0:05/)).toBeInTheDocument();
    expect(screen.getByText(/0:59/)).toBeInTheDocument();
    expect(screen.getByText(/1:00/)).toBeInTheDocument();
    expect(screen.getByText(/2:05/)).toBeInTheDocument();
  });

  it('does not call handlers when not provided', async () => {
    const user = userEvent.setup();
    render(<WhisperList whispers={mockWhispers} />);

    const deleteButton = screen.getByTestId('delete-button-0');
    await user.click(deleteButton);

    const playButton = screen.getByTestId('play-button-0');
    await user.click(playButton);

    const noteButton = screen.getAllByTestId('convert-to-note')[0];
    await user.click(noteButton);

    // Should not throw errors
    expect(true).toBe(true);
  });
});
