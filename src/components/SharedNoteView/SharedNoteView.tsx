import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useParams, useNavigate } from 'react-router-dom';
import './SharedNoteView.css';

interface SharedNote {
  note_id: string;
  title: string;
  content: string | null;
  created_at: string;
  updated_at: string;
}

const SharedNoteView: React.FC = () => {
  const { token } = useParams<{ token: string }>();
  const navigate = useNavigate();
  const [note, setNote] = useState<SharedNote | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (token) {
      loadSharedNote(token);
    } else {
      setError('Invalid share link');
      setLoading(false);
    }
  }, [token]);

  const loadSharedNote = async (shareToken: string) => {
    try {
      setLoading(true);
      setError(null);
      const sharedNote = await invoke<SharedNote>('get_shared_note', { token: shareToken });
      setNote(sharedNote);
    } catch (err) {
      console.error('Failed to load shared note:', err);
      setError(String(err) || 'Failed to load note. The link may have expired or is invalid.');
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleString();
  };

  const goHome = () => {
    navigate('/');
  };

  if (loading) {
    return (
      <div className="shared-note-view">
        <div className="loading-container">
          <div className="spinner"></div>
          <p>Loading shared note...</p>
        </div>
      </div>
    );
  }

  if (error || !note) {
    return (
      <div className="shared-note-view">
        <div className="error-container">
          <div className="error-icon">⚠️</div>
          <h2>Unable to Load Note</h2>
          <p className="error-text">{error || 'This note is not available.'}</p>
          <button className="home-button" onClick={goHome}>
            Go to Home
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="shared-note-view">
      <div className="shared-note-container">
        <div className="shared-note-header">
          <div className="header-content">
            <h1 className="note-title">{note.title}</h1>
            <div className="note-metadata">
              <span className="metadata-item">Created: {formatDate(note.created_at)}</span>
              <span className="metadata-separator">•</span>
              <span className="metadata-item">Updated: {formatDate(note.updated_at)}</span>
            </div>
          </div>
          <button className="home-button-top" onClick={goHome}>
            View in App
          </button>
        </div>

        <div className="shared-note-content">
          {note.content ? (
            <div className="note-text">{note.content}</div>
          ) : (
            <div className="no-content">This note has no content.</div>
          )}
        </div>

        <div className="shared-note-footer">
          <p className="footer-text">
            This note was shared with you via HiNotes.
            <a href="/" className="footer-link">Learn more about HiNotes</a>
          </p>
        </div>
      </div>
    </div>
  );
};

export default SharedNoteView;
