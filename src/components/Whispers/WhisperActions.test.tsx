import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { WhisperActions } from './WhisperActions';

// Mock Arco Design icons
vi.mock('@arco-design/web-react/icon', () => ({
  IconFile: () => <span>File Icon</span>,
  IconCheckCircle: () => <span>Check Icon</span>,
  IconCalendar: () => <span>Calendar Icon</span>,
}));

describe('WhisperActions Component', () => {
  const mockOnConvert = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders all action buttons', () => {
    render(<WhisperActions whisperId="test-123" />);

    expect(screen.getByTestId('convert-to-note')).toBeInTheDocument();
    expect(screen.getByTestId('convert-to-todo')).toBeInTheDocument();
    expect(screen.getByTestId('convert-to-calendar')).toBeInTheDocument();
  });

  it('displays button labels', () => {
    render(<WhisperActions whisperId="test-123" />);

    expect(screen.getByText('Note')).toBeInTheDocument();
    expect(screen.getByText('To-Do')).toBeInTheDocument();
    expect(screen.getByText('Calendar')).toBeInTheDocument();
  });

  it('calls onConvert with "note" when Note button clicked', async () => {
    const user = userEvent.setup();
    render(<WhisperActions whisperId="test-123" onConvert={mockOnConvert} />);

    const noteButton = screen.getByTestId('convert-to-note');
    await user.click(noteButton);

    expect(mockOnConvert).toHaveBeenCalledWith('note');
    expect(mockOnConvert).toHaveBeenCalledTimes(1);
  });

  it('calls onConvert with "todo" when To-Do button clicked', async () => {
    const user = userEvent.setup();
    render(<WhisperActions whisperId="test-123" onConvert={mockOnConvert} />);

    const todoButton = screen.getByTestId('convert-to-todo');
    await user.click(todoButton);

    expect(mockOnConvert).toHaveBeenCalledWith('todo');
    expect(mockOnConvert).toHaveBeenCalledTimes(1);
  });

  it('calls onConvert with "calendar" when Calendar button clicked', async () => {
    const user = userEvent.setup();
    render(<WhisperActions whisperId="test-123" onConvert={mockOnConvert} />);

    const calendarButton = screen.getByTestId('convert-to-calendar');
    await user.click(calendarButton);

    expect(mockOnConvert).toHaveBeenCalledWith('calendar');
    expect(mockOnConvert).toHaveBeenCalledTimes(1);
  });

  it('disables all buttons when disabled prop is true', () => {
    render(<WhisperActions whisperId="test-123" disabled={true} />);

    expect(screen.getByTestId('convert-to-note')).toBeDisabled();
    expect(screen.getByTestId('convert-to-todo')).toBeDisabled();
    expect(screen.getByTestId('convert-to-calendar')).toBeDisabled();
  });

  it('enables all buttons when disabled prop is false', () => {
    render(<WhisperActions whisperId="test-123" disabled={false} />);

    expect(screen.getByTestId('convert-to-note')).not.toBeDisabled();
    expect(screen.getByTestId('convert-to-todo')).not.toBeDisabled();
    expect(screen.getByTestId('convert-to-calendar')).not.toBeDisabled();
  });

  it('does not call onConvert when buttons are disabled', () => {
    render(<WhisperActions whisperId="test-123" onConvert={mockOnConvert} disabled={true} />);

    const noteButton = screen.getByTestId('convert-to-note');

    // Verify button is disabled (cannot be clicked)
    expect(noteButton).toBeDisabled();
    expect(mockOnConvert).not.toHaveBeenCalled();
  });

  it('displays icons on buttons', () => {
    render(<WhisperActions whisperId="test-123" />);

    const noteButton = screen.getByTestId('convert-to-note');
    const todoButton = screen.getByTestId('convert-to-todo');
    const calendarButton = screen.getByTestId('convert-to-calendar');

    // Check that buttons exist with text content (icons are mocked as spans)
    expect(noteButton).toBeInTheDocument();
    expect(todoButton).toBeInTheDocument();
    expect(calendarButton).toBeInTheDocument();
    expect(screen.getByText('File Icon')).toBeInTheDocument();
    expect(screen.getByText('Check Icon')).toBeInTheDocument();
    expect(screen.getByText('Calendar Icon')).toBeInTheDocument();
  });

  it('uses correct data-testid attribute', () => {
    render(<WhisperActions whisperId="whisper-456" />);

    expect(screen.getByTestId('whisper-actions-whisper-456')).toBeInTheDocument();
  });

  it('handles multiple clicks on same button', async () => {
    const user = userEvent.setup();
    render(<WhisperActions whisperId="test-123" onConvert={mockOnConvert} />);

    const noteButton = screen.getByTestId('convert-to-note');
    await user.click(noteButton);
    await user.click(noteButton);
    await user.click(noteButton);

    expect(mockOnConvert).toHaveBeenCalledTimes(3);
    expect(mockOnConvert).toHaveBeenCalledWith('note');
  });

  it('handles clicks on different buttons', async () => {
    const user = userEvent.setup();
    render(<WhisperActions whisperId="test-123" onConvert={mockOnConvert} />);

    const noteButton = screen.getByTestId('convert-to-note');
    const todoButton = screen.getByTestId('convert-to-todo');
    const calendarButton = screen.getByTestId('convert-to-calendar');

    await user.click(noteButton);
    await user.click(todoButton);
    await user.click(calendarButton);

    expect(mockOnConvert).toHaveBeenCalledTimes(3);
    expect(mockOnConvert).toHaveBeenNthCalledWith(1, 'note');
    expect(mockOnConvert).toHaveBeenNthCalledWith(2, 'todo');
    expect(mockOnConvert).toHaveBeenNthCalledWith(3, 'calendar');
  });

  it('does not error when onConvert is not provided', async () => {
    const user = userEvent.setup();
    render(<WhisperActions whisperId="test-123" />);

    const noteButton = screen.getByTestId('convert-to-note');
    await user.click(noteButton);

    // Should not throw errors
    expect(true).toBe(true);
  });

  it('renders with correct button style (outline)', () => {
    render(<WhisperActions whisperId="test-123" />);

    const noteButton = screen.getByTestId('convert-to-note');
    const todoButton = screen.getByTestId('convert-to-todo');
    const calendarButton = screen.getByTestId('convert-to-calendar');

    expect(noteButton).toHaveClass('arco-btn-outline');
    expect(todoButton).toHaveClass('arco-btn-outline');
    expect(calendarButton).toHaveClass('arco-btn-outline');
  });

  it('renders with small size', () => {
    render(<WhisperActions whisperId="test-123" />);

    const noteButton = screen.getByTestId('convert-to-note');
    const todoButton = screen.getByTestId('convert-to-todo');
    const calendarButton = screen.getByTestId('convert-to-calendar');

    expect(noteButton).toHaveClass('arco-btn-size-small');
    expect(todoButton).toHaveClass('arco-btn-size-small');
    expect(calendarButton).toHaveClass('arco-btn-size-small');
  });
});
