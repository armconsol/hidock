// Whisper note types - quick voice recordings

export interface Whisper {
  id: string;
  content: string; // Transcribed text
  audioUrl: string;
  duration: number; // Duration in seconds
  createdAt: Date;
  updatedAt: Date;
}

export interface WhisperFilter {
  searchQuery?: string;
  sortField?: 'create_time' | 'duration';
}

export interface WhisperConvertOptions {
  targetType: 'note' | 'todo' | 'calendar';
  whisperId: string;
}
