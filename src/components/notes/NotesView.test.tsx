import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { NotesView } from './NotesView';
import { NoteCard } from './NoteCard';
import { NotesList } from './NotesList';
import { NoteDetail } from './NoteDetail';
import { useNotesStore } from '../../store/notesStore';
import type { Note } from '../../types/notes';

// Mock Ant Design icons
vi.mock('@ant-design/icons', () => ({
  SearchOutlined: () => <span>Search Icon</span>,
  PlusOutlined: () => <span>Plus Icon</span>,
  FilterOutlined: () => <span>Filter Icon</span>,
  SortAscendingOutlined: () => <span>Sort Icon</span>,
  PushpinOutlined: () => <span>Pin Icon</span>,
  ClockCircleOutlined: () => <span>Clock Icon</span>,
  MoreOutlined: () => <span>More Icon</span>,
  DeleteOutlined: () => <span>Delete Icon</span>,
  InboxOutlined: () => <span>Archive Icon</span>,
  TagOutlined: () => <span>Tag Icon</span>,
  SaveOutlined: () => <span>Save Icon</span>,
}));

// Note: message API mock is handled globally in src/test/setup.ts
// to prevent ReactDOM.render() errors in React 19

describe('NoteCard', () => {
  const mockNote: Note = {
    id: '1',
    title: 'Test Note',
    content: 'Test content',
    createdAt: new Date('2025-01-01'),
    updatedAt: new Date('2025-01-02'),
    duration: 125, // 2:05
    tags: ['work', 'important'],
    isPinned: false,
    isArchived: false,
  };

  it('renders note title', () => {
    render(<NoteCard note={mockNote} />);
    expect(screen.getByText('Test Note')).toBeInTheDocument();
  });

  it('displays duration when present', () => {
    render(<NoteCard note={mockNote} />);
    expect(screen.getByText('2:05')).toBeInTheDocument();
  });

  it('shows pin icon for pinned notes', () => {
    const pinnedNote = { ...mockNote, isPinned: true };
    render(<NoteCard note={pinnedNote} />);
    expect(screen.getByText('Pin Icon')).toBeInTheDocument();
  });

  it('displays tags (max 3)', () => {
    render(<NoteCard note={mockNote} />);
    expect(screen.getByText('work')).toBeInTheDocument();
    expect(screen.getByText('important')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const handleClick = vi.fn();
    render(<NoteCard note={mockNote} onClick={handleClick} />);

    const card = screen.getByText('Test Note').closest('.note-card');
    await userEvent.click(card!);

    expect(handleClick).toHaveBeenCalledOnce();
  });

  it('applies selected class when isSelected is true', () => {
    const { container } = render(<NoteCard note={mockNote} isSelected />);
    expect(container.querySelector('.note-card-selected')).toBeInTheDocument();
  });
});

describe('NotesList', () => {
  beforeEach(() => {
    useNotesStore.setState({
      notes: [],
      selectedNoteId: null,
      filter: {},
      sortBy: 'updatedAt',
      sortOrder: 'desc',
    });
  });

  it('renders empty state when no notes', () => {
    render(<NotesList />);
    expect(
      screen.getByText('No notes yet. Create your first note!')
    ).toBeInTheDocument();
  });

  it('renders search input', () => {
    render(<NotesList />);
    expect(screen.getByPlaceholderText('Search notes...')).toBeInTheDocument();
  });

  it('renders new note button', () => {
    render(<NotesList />);
    expect(screen.getByText('New Note')).toBeInTheDocument();
  });

  it('creates new note when button clicked', async () => {
    const initialCount = useNotesStore.getState().notes.length;

    render(<NotesList />);

    const newNoteButton = screen.getByText('New Note');
    await userEvent.click(newNoteButton);

    await waitFor(() => {
      const finalCount = useNotesStore.getState().notes.length;
      expect(finalCount).toBe(initialCount + 1);
    });
  });

  it('filters notes by search query', async () => {
    const mockNotes: Note[] = [
      {
        id: '1',
        title: 'JavaScript Tutorial',
        content: 'Learn JS',
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      {
        id: '2',
        title: 'Python Guide',
        content: 'Learn Python',
        createdAt: new Date(),
        updatedAt: new Date(),
      },
    ];

    useNotesStore.setState({ notes: mockNotes });

    render(<NotesList />);

    const searchInput = screen.getByPlaceholderText('Search notes...');
    await userEvent.type(searchInput, 'JavaScript');

    await waitFor(() => {
      expect(screen.getByText('JavaScript Tutorial')).toBeInTheDocument();
      expect(screen.queryByText('Python Guide')).not.toBeInTheDocument();
    });
  });

  it('displays note cards for each note', () => {
    const mockNotes: Note[] = [
      {
        id: '1',
        title: 'Note 1',
        content: 'Content 1',
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      {
        id: '2',
        title: 'Note 2',
        content: 'Content 2',
        createdAt: new Date(),
        updatedAt: new Date(),
      },
    ];

    useNotesStore.setState({ notes: mockNotes });

    render(<NotesList />);

    expect(screen.getByText('Note 1')).toBeInTheDocument();
    expect(screen.getByText('Note 2')).toBeInTheDocument();
  });
});

describe('NoteDetail', () => {
  beforeEach(() => {
    useNotesStore.setState({
      notes: [],
      selectedNoteId: null,
      filter: {},
      sortBy: 'updatedAt',
      sortOrder: 'desc',
    });
  });

  it('shows empty state when no note selected', () => {
    render(<NoteDetail />);
    expect(
      screen.getByText('Select a note to view or create a new one')
    ).toBeInTheDocument();
  });

  it('displays selected note', () => {
    const mockNote: Note = {
      id: '1',
      title: 'Test Note',
      content: 'Test content',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    useNotesStore.setState({
      notes: [mockNote],
      selectedNoteId: '1',
    });

    render(<NoteDetail />);

    expect(screen.getByDisplayValue('Test Note')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Test content')).toBeInTheDocument();
  });

  it('shows save button when changes are made', async () => {
    const mockNote: Note = {
      id: '1',
      title: 'Test Note',
      content: 'Test content',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    useNotesStore.setState({
      notes: [mockNote],
      selectedNoteId: '1',
    });

    render(<NoteDetail />);

    const titleInput = screen.getByDisplayValue('Test Note');
    await userEvent.clear(titleInput);
    await userEvent.type(titleInput, 'Updated Title');

    await waitFor(() => {
      expect(screen.getByText('Save')).toBeInTheDocument();
    });
  });

  it('saves note when save button clicked', async () => {
    const mockNote: Note = {
      id: '1',
      title: 'Test Note',
      content: 'Test content',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    useNotesStore.setState({
      notes: [mockNote],
      selectedNoteId: '1',
    });

    const updateNoteSpy = vi.spyOn(useNotesStore.getState(), 'updateNote');

    render(<NoteDetail />);

    const titleInput = screen.getByDisplayValue('Test Note');
    await userEvent.clear(titleInput);
    await userEvent.type(titleInput, 'Updated Title');

    // Wait for save button to appear
    await waitFor(() => {
      expect(screen.getByText('Save')).toBeInTheDocument();
    });

    // Click save button
    const saveButton = screen.getByText('Save');
    await userEvent.click(saveButton);

    // Verify updateNote was called correctly
    await waitFor(() => {
      expect(updateNoteSpy).toHaveBeenCalledWith('1', {
        title: 'Updated Title',
        content: 'Test content',
      });
    });
  });

  it('displays tags when present', () => {
    const mockNote: Note = {
      id: '1',
      title: 'Test Note',
      content: 'Test content',
      createdAt: new Date(),
      updatedAt: new Date(),
      tags: ['urgent', 'work'],
    };

    useNotesStore.setState({
      notes: [mockNote],
      selectedNoteId: '1',
    });

    render(<NoteDetail />);

    expect(screen.getByText('urgent')).toBeInTheDocument();
    expect(screen.getByText('work')).toBeInTheDocument();
  });
});

describe('NotesView', () => {
  it('renders both list and detail views', () => {
    render(<NotesView />);

    // Should render the main container
    const notesView = document.querySelector('.notes-view');
    expect(notesView).toBeInTheDocument();

    // Should have sidebar and content areas
    expect(document.querySelector('.notes-view-sidebar')).toBeInTheDocument();
    expect(document.querySelector('.notes-view-content')).toBeInTheDocument();
  });
});

describe('useNotesStore', () => {
  beforeEach(() => {
    useNotesStore.setState({
      notes: [],
      selectedNoteId: null,
      filter: {},
      sortBy: 'updatedAt',
      sortOrder: 'desc',
    });
  });

  it('adds new note', () => {
    const { addNote } = useNotesStore.getState();

    addNote({
      title: 'New Note',
      content: 'New content',
    });

    const { notes } = useNotesStore.getState();

    expect(notes).toHaveLength(1);
    expect(notes[0].title).toBe('New Note');
    expect(notes[0].content).toBe('New content');
    expect(notes[0].id).toBeDefined();
  });

  it('updates note', () => {
    const { addNote, updateNote } = useNotesStore.getState();

    addNote({ title: 'Original', content: 'Content' });

    const { notes } = useNotesStore.getState();
    const noteId = notes[0].id;

    updateNote(noteId, { title: 'Updated' });

    const updatedState = useNotesStore.getState();
    expect(updatedState.notes[0].title).toBe('Updated');
  });

  it('deletes note', () => {
    const { addNote, deleteNote } = useNotesStore.getState();

    addNote({ title: 'To Delete', content: 'Content' });

    const { notes } = useNotesStore.getState();
    const noteId = notes[0].id;

    deleteNote(noteId);

    const updatedState = useNotesStore.getState();
    expect(updatedState.notes).toHaveLength(0);
  });

  it('toggles pin status', () => {
    const { addNote, togglePinNote } = useNotesStore.getState();

    addNote({ title: 'Note', content: 'Content' });

    const { notes } = useNotesStore.getState();
    const noteId = notes[0].id;

    togglePinNote(noteId);

    let updatedState = useNotesStore.getState();
    expect(updatedState.notes[0].isPinned).toBe(true);

    togglePinNote(noteId);

    updatedState = useNotesStore.getState();
    expect(updatedState.notes[0].isPinned).toBe(false);
  });

  it('filters notes by search query', () => {
    const { addNote, setFilter, getFilteredNotes } = useNotesStore.getState();

    addNote({ title: 'JavaScript', content: 'JS content' });
    addNote({ title: 'Python', content: 'Py content' });

    setFilter({ searchQuery: 'java' });

    const filtered = getFilteredNotes();

    expect(filtered).toHaveLength(1);
    expect(filtered[0].title).toBe('JavaScript');
  });

  it('sorts notes by title', () => {
    const { addNote, setSorting, getFilteredNotes } = useNotesStore.getState();

    addNote({ title: 'Zebra', content: '' });
    addNote({ title: 'Apple', content: '' });

    setSorting('title', 'asc');

    const sorted = getFilteredNotes();

    expect(sorted[0].title).toBe('Apple');
    expect(sorted[1].title).toBe('Zebra');
  });

  it('should mark note as favorite', () => {
    const { addNote, togglePinNote } = useNotesStore.getState();

    addNote({ title: 'Test Note', content: 'Content' });

    const { notes } = useNotesStore.getState();
    const noteId = notes[0].id;

    // Pin the note (mark as favorite)
    togglePinNote(noteId);

    const updatedState = useNotesStore.getState();
    expect(updatedState.notes[0].isPinned).toBe(true);
  });
});
