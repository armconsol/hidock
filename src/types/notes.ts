// Note types

export interface Note {
  id: string;
  title: string;
  content: string;
  createdAt: Date;
  updatedAt: Date;
  duration?: number; // Duration in seconds for audio notes
  tags?: string[];
  isPinned?: boolean;
  isArchived?: boolean;
}

export interface NoteFilter {
  searchQuery?: string;
  tags?: string[];
  isPinned?: boolean;
  isArchived?: boolean;
}

export type NoteSortBy = 'createdAt' | 'updatedAt' | 'title';
export type SortOrder = 'asc' | 'desc';
