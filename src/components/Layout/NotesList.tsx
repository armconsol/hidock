import { Card, Empty, Space } from 'antd';
import { ClockCircleOutlined, CalendarOutlined } from '@ant-design/icons';
import { useNotesStore } from '../../store/notesStore';
import { useNavigate } from 'react-router-dom';
import type { Note } from '../../types/notes';
import './NotesList.css';

interface NotesListProps {
  showArchived?: boolean;
  limit?: number;
}

export function NotesList({ showArchived = false, limit }: NotesListProps) {
  const navigate = useNavigate();
  const { getFilteredNotes, selectNote } = useNotesStore();

  const allNotes = getFilteredNotes();
  const displayNotes = limit ? allNotes.slice(0, limit) : allNotes;

  const handleNoteClick = (note: Note) => {
    selectNote(note.id);
    navigate(`/notes/${note.id}`);
  };

  const formatDate = (date: Date) => {
    const noteDate = new Date(date);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - noteDate.getTime());
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return 'Today';
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return `${diffDays} days ago`;
    } else {
      return noteDate.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: noteDate.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
      });
    }
  };

  const formatDuration = (seconds?: number) => {
    if (!seconds) return null;

    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;

    if (minutes === 0) {
      return `${remainingSeconds}s`;
    } else {
      return `${minutes}m ${remainingSeconds}s`;
    }
  };

  if (displayNotes.length === 0) {
    return (
      <div className="notes-list-empty">
        <Empty
          description={
            showArchived
              ? 'No archived notes'
              : 'No notes yet. Start recording or create a new note!'
          }
        />
      </div>
    );
  }

  return (
    <div className="notes-list-container">
      <Space direction="vertical" size="medium" className="notes-list">
        {displayNotes.map((note) => (
          <Card
            key={note.id}
            className="note-card"
            hoverable
            onClick={() => handleNoteClick(note)}
            bordered
          >
            <div className="note-card-content">
              <div className="note-card-main">
                <h3 className="note-title">{note.title || 'Untitled Note'}</h3>
                <div className="note-meta">
                  <span className="note-meta-item">
                    <CalendarOutlined className="note-meta-icon" />
                    {formatDate(note.updatedAt)}
                  </span>
                  {note.duration && (
                    <span className="note-meta-item">
                      <ClockCircleOutlined className="note-meta-icon" />
                      {formatDuration(note.duration)}
                    </span>
                  )}
                </div>
              </div>
              {note.tags && note.tags.length > 0 && (
                <div className="note-tags">
                  {note.tags.slice(0, 3).map((tag, index) => (
                    <span key={index} className="note-tag">
                      {tag}
                    </span>
                  ))}
                  {note.tags.length > 3 && (
                    <span className="note-tag-more">+{note.tags.length - 3}</span>
                  )}
                </div>
              )}
            </div>
          </Card>
        ))}
      </Space>
    </div>
  );
}
