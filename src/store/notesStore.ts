import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Note, NoteFilter, NoteSortBy, SortOrder } from '../types/notes';

interface NotesState {
  notes: Note[];
  selectedNoteId: string | null;
  filter: NoteFilter;
  sortBy: NoteSortBy;
  sortOrder: SortOrder;

  // Actions
  addNote: (note: Omit<Note, 'id' | 'createdAt' | 'updatedAt'>) => void;
  updateNote: (id: string, updates: Partial<Note>) => void;
  deleteNote: (id: string) => void;
  selectNote: (id: string | null) => void;
  setFilter: (filter: NoteFilter) => void;
  setSorting: (sortBy: NoteSortBy, sortOrder: SortOrder) => void;
  togglePinNote: (id: string) => void;
  toggleArchiveNote: (id: string) => void;
  getFilteredNotes: () => Note[];
}

export const useNotesStore = create<NotesState>()(
  persist(
    (set, get) => ({
      notes: [],
      selectedNoteId: null,
      filter: {},
      sortBy: 'updatedAt',
      sortOrder: 'desc',

      addNote: (noteData) => {
        const newNote: Note = {
          ...noteData,
          id: crypto.randomUUID(),
          createdAt: new Date(),
          updatedAt: new Date(),
        };
        set((state) => ({
          notes: [newNote, ...state.notes],
          selectedNoteId: newNote.id,
        }));
      },

      updateNote: (id, updates) => {
        set((state) => ({
          notes: state.notes.map((note) =>
            note.id === id
              ? { ...note, ...updates, updatedAt: new Date() }
              : note
          ),
        }));
      },

      deleteNote: (id) => {
        set((state) => ({
          notes: state.notes.filter((note) => note.id !== id),
          selectedNoteId: state.selectedNoteId === id ? null : state.selectedNoteId,
        }));
      },

      selectNote: (id) => {
        set({ selectedNoteId: id });
      },

      setFilter: (filter) => {
        set({ filter });
      },

      setSorting: (sortBy, sortOrder) => {
        set({ sortBy, sortOrder });
      },

      togglePinNote: (id) => {
        set((state) => ({
          notes: state.notes.map((note) =>
            note.id === id ? { ...note, isPinned: !note.isPinned } : note
          ),
        }));
      },

      toggleArchiveNote: (id) => {
        set((state) => ({
          notes: state.notes.map((note) =>
            note.id === id ? { ...note, isArchived: !note.isArchived } : note
          ),
        }));
      },

      getFilteredNotes: () => {
        const { notes, filter, sortBy, sortOrder } = get();

        let filtered = notes.filter((note) => {
          // Filter by search query
          if (filter.searchQuery) {
            const query = filter.searchQuery.toLowerCase();
            if (
              !note.title.toLowerCase().includes(query) &&
              !note.content.toLowerCase().includes(query)
            ) {
              return false;
            }
          }

          // Filter by tags
          if (filter.tags && filter.tags.length > 0) {
            if (!note.tags || !filter.tags.some((tag) => note.tags?.includes(tag))) {
              return false;
            }
          }

          // Filter by pinned status
          if (filter.isPinned !== undefined && note.isPinned !== filter.isPinned) {
            return false;
          }

          // Filter by archived status
          if (filter.isArchived !== undefined && note.isArchived !== filter.isArchived) {
            return false;
          }

          return true;
        });

        // Sort
        filtered.sort((a, b) => {
          let aValue: string | number | Date;
          let bValue: string | number | Date;

          if (sortBy === 'title') {
            aValue = a.title.toLowerCase();
            bValue = b.title.toLowerCase();
          } else {
            aValue = new Date(a[sortBy]).getTime();
            bValue = new Date(b[sortBy]).getTime();
          }

          if (sortOrder === 'asc') {
            return aValue > bValue ? 1 : -1;
          } else {
            return aValue < bValue ? 1 : -1;
          }
        });

        return filtered;
      },
    }),
    {
      name: 'notes-storage',
    }
  )
);
