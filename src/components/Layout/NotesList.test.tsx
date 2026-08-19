import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { NotesList } from './NotesList';
import { useNotesStore } from '../../store/notesStore';
import type { Note } from '../../types/notes';

// Mock the notes store
vi.mock('../../store/notesStore', () => ({
  useNotesStore: vi.fn(),
}));

// Mock react-router-dom
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe('NotesList', () => {
  const mockGetFilteredNotes = vi.fn();
  const mockSelectNote = vi.fn();

  const mockNotes: Note[] = [
    {
      id: '1',
      title: 'First Note',
      content: 'Content of first note',
      createdAt: new Date('2024-01-01'),
      updatedAt: new Date('2024-01-01'),
      duration: 120,
      tags: ['work', 'important'],
    },
    {
      id: '2',
      title: 'Second Note',
      content: 'Content of second note',
      createdAt: new Date('2024-01-02'),
      updatedAt: new Date('2024-01-02'),
      tags: ['personal'],
    },
    {
      id: '3',
      title: 'Third Note',
      content: 'Content of third note',
      createdAt: new Date('2024-01-03'),
      updatedAt: new Date('2024-01-03'),
      duration: 45,
      tags: ['work', 'meeting', 'client', 'urgent'],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    mockNavigate.mockClear();

    (useNotesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      getFilteredNotes: mockGetFilteredNotes,
      selectNote: mockSelectNote,
    });
  });

  it('renders notes list with multiple notes', () => {
    mockGetFilteredNotes.mockReturnValue(mockNotes);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(screen.getByText('First Note')).toBeInTheDocument();
    expect(screen.getByText('Second Note')).toBeInTheDocument();
    expect(screen.getByText('Third Note')).toBeInTheDocument();
  });

  it('displays empty state when no notes exist', () => {
    mockGetFilteredNotes.mockReturnValue([]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(
      screen.getByText(/No notes yet. Start recording or create a new note!/i)
    ).toBeInTheDocument();
  });

  it('displays archived empty state when showArchived is true', () => {
    mockGetFilteredNotes.mockReturnValue([]);

    render(
      <BrowserRouter>
        <NotesList showArchived={true} />
      </BrowserRouter>
    );

    expect(screen.getByText('No archived notes')).toBeInTheDocument();
  });

  it('displays duration for notes with duration', () => {
    mockGetFilteredNotes.mockReturnValue([mockNotes[0]]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(screen.getByText('2m 0s')).toBeInTheDocument();
  });

  it('formats duration correctly for different values', () => {
    const noteWithShortDuration: Note = {
      ...mockNotes[0],
      duration: 45,
    };

    mockGetFilteredNotes.mockReturnValue([noteWithShortDuration]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(screen.getByText('45s')).toBeInTheDocument();
  });

  it('displays note tags', () => {
    mockGetFilteredNotes.mockReturnValue([mockNotes[0]]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(screen.getByText('work')).toBeInTheDocument();
    expect(screen.getByText('important')).toBeInTheDocument();
  });

  it('truncates tags to 3 and shows count for additional tags', () => {
    mockGetFilteredNotes.mockReturnValue([mockNotes[2]]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(screen.getByText('work')).toBeInTheDocument();
    expect(screen.getByText('meeting')).toBeInTheDocument();
    expect(screen.getByText('client')).toBeInTheDocument();
    expect(screen.getByText('+1')).toBeInTheDocument();
  });

  it('navigates to note detail when note card is clicked', async () => {
    const user = userEvent.setup();
    mockGetFilteredNotes.mockReturnValue([mockNotes[0]]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    const noteCard = screen.getByText('First Note').closest('.arco-card');
    if (noteCard) {
      await user.click(noteCard);
    }

    expect(mockSelectNote).toHaveBeenCalledWith('1');
    expect(mockNavigate).toHaveBeenCalledWith('/notes/1');
  });

  it('limits notes displayed when limit prop is provided', () => {
    mockGetFilteredNotes.mockReturnValue(mockNotes);

    render(
      <BrowserRouter>
        <NotesList limit={2} />
      </BrowserRouter>
    );

    expect(screen.getByText('First Note')).toBeInTheDocument();
    expect(screen.getByText('Second Note')).toBeInTheDocument();
    expect(screen.queryByText('Third Note')).not.toBeInTheDocument();
  });

  it('displays untitled for notes without title', () => {
    const untitledNote: Note = {
      ...mockNotes[0],
      title: '',
    };
    mockGetFilteredNotes.mockReturnValue([untitledNote]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    expect(screen.getByText('Untitled Note')).toBeInTheDocument();
  });

  it('formats dates correctly for recent notes', () => {
    // Create a date that's clearly in the recent past
    const recentDate = new Date();
    recentDate.setDate(recentDate.getDate() - 2);

    const recentNote: Note = {
      ...mockNotes[0],
      updatedAt: recentDate,
    };
    mockGetFilteredNotes.mockReturnValue([recentNote]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    // Check that a relative date is displayed (e.g., "2 days ago", "Yesterday")
    // The exact text depends on timing, but should match the pattern
    const dateText = screen.getByText(/days? ago|Yesterday|Today/);
    expect(dateText).toBeInTheDocument();
  });

  it('does not display duration for notes without duration', () => {
    mockGetFilteredNotes.mockReturnValue([mockNotes[1]]);

    render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    // Should only have the date icon, not the clock icon
    const icons = document.querySelectorAll('.note-meta-icon');
    expect(icons).toHaveLength(1);
  });

  it('renders note cards with proper structure', () => {
    mockGetFilteredNotes.mockReturnValue([mockNotes[0]]);

    const { container } = render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    const noteCard = container.querySelector('.note-card');
    expect(noteCard).toBeInTheDocument();
    expect(noteCard?.querySelector('.note-card-content')).toBeInTheDocument();
    expect(noteCard?.querySelector('.note-title')).toBeInTheDocument();
    expect(noteCard?.querySelector('.note-meta')).toBeInTheDocument();
  });

  it('applies hoverable class to note cards', () => {
    mockGetFilteredNotes.mockReturnValue([mockNotes[0]]);

    const { container } = render(
      <BrowserRouter>
        <NotesList />
      </BrowserRouter>
    );

    const noteCard = container.querySelector('.arco-card-hoverable');
    expect(noteCard).toBeInTheDocument();
  });
});
