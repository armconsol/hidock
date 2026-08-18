import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './ShareDialog.css';

interface ShareDialogProps {
  isOpen: boolean;
  onClose: () => void;
  noteId: string;
  noteTitle: string;
}

interface ShareLink {
  id: string;
  note_id: string;
  token: string;
  expires_at: string | null;
  created_at: string;
  last_accessed_at: string | null;
  access_count: number;
}

const ShareDialog: React.FC<ShareDialogProps> = ({ isOpen, onClose, noteId, noteTitle }) => {
  const [shareLinks, setShareLinks] = useState<ShareLink[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expirationDays, setExpirationDays] = useState<number>(7);
  const [copiedToken, setCopiedToken] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadShareLinks();
    }
  }, [isOpen, noteId]);

  const loadShareLinks = async () => {
    try {
      setLoading(true);
      setError(null);
      const links = await invoke<ShareLink[]>('list_share_links', { noteId });
      setShareLinks(links);
    } catch (err) {
      console.error('Failed to load share links:', err);
      setError(`Failed to load share links: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const generateShareLink = async () => {
    try {
      setLoading(true);
      setError(null);
      const expiresInDays = expirationDays > 0 ? expirationDays : null;
      await invoke<ShareLink>('create_share_link', {
        noteId,
        expiresInDays,
      });
      await loadShareLinks();
    } catch (err) {
      console.error('Failed to create share link:', err);
      setError(`Failed to create share link: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const deleteShareLink = async (shareId: string) => {
    try {
      setLoading(true);
      setError(null);
      await invoke('delete_share_link', { shareId });
      await loadShareLinks();
    } catch (err) {
      console.error('Failed to delete share link:', err);
      setError(`Failed to delete share link: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = async (token: string) => {
    // Build the full URL - in production this would be your actual domain
    const shareUrl = `${window.location.origin}/shared/${token}`;

    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopiedToken(token);
      setTimeout(() => setCopiedToken(null), 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
      setError('Failed to copy link to clipboard');
    }
  };

  const formatDate = (dateString: string | null): string => {
    if (!dateString) return 'Never';
    return new Date(dateString).toLocaleDateString();
  };

  const isExpired = (expiresAt: string | null): boolean => {
    if (!expiresAt) return false;
    return new Date(expiresAt) < new Date();
  };

  if (!isOpen) return null;

  return (
    <div className="share-dialog-overlay" onClick={onClose}>
      <div className="share-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Share Note</h2>
          <button className="close-button" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        <div className="dialog-content">
          <div className="note-info">
            <h3>{noteTitle}</h3>
            <p className="note-id">Note ID: {noteId}</p>
          </div>

          <div className="generate-section">
            <h4>Generate New Share Link</h4>
            <div className="expiration-control">
              <label htmlFor="expiration-days">Expires in:</label>
              <select
                id="expiration-days"
                value={expirationDays}
                onChange={(e) => setExpirationDays(Number(e.target.value))}
                disabled={loading}
              >
                <option value="1">1 day</option>
                <option value="7">7 days</option>
                <option value="30">30 days</option>
                <option value="90">90 days</option>
                <option value="0">Never</option>
              </select>
            </div>
            <button
              className="generate-button"
              onClick={generateShareLink}
              disabled={loading}
            >
              {loading ? 'Generating...' : 'Generate Link'}
            </button>
          </div>

          {error && <div className="error-message">{error}</div>}

          <div className="share-links-section">
            <h4>Active Share Links</h4>
            {shareLinks.length === 0 ? (
              <p className="no-links">No share links created yet</p>
            ) : (
              <div className="share-links-list">
                {shareLinks.map((link) => (
                  <div
                    key={link.id}
                    className={`share-link-item ${isExpired(link.expires_at) ? 'expired' : ''}`}
                  >
                    <div className="link-info">
                      <div className="link-token">
                        <code>{link.token.substring(0, 16)}...</code>
                        <button
                          className="copy-button"
                          onClick={() => copyToClipboard(link.token)}
                          disabled={isExpired(link.expires_at)}
                        >
                          {copiedToken === link.token ? '✓ Copied' : '📋 Copy Link'}
                        </button>
                      </div>
                      <div className="link-metadata">
                        <span className="metadata-item">
                          Created: {formatDate(link.created_at)}
                        </span>
                        {link.expires_at && (
                          <span
                            className={`metadata-item ${isExpired(link.expires_at) ? 'expired-text' : ''}`}
                          >
                            Expires: {formatDate(link.expires_at)}
                            {isExpired(link.expires_at) && ' (Expired)'}
                          </span>
                        )}
                        {!link.expires_at && (
                          <span className="metadata-item">No expiration</span>
                        )}
                        <span className="metadata-item">
                          Accessed: {link.access_count} times
                        </span>
                        {link.last_accessed_at && (
                          <span className="metadata-item">
                            Last: {formatDate(link.last_accessed_at)}
                          </span>
                        )}
                      </div>
                    </div>
                    <button
                      className="delete-button"
                      onClick={() => deleteShareLink(link.id)}
                      disabled={loading}
                      aria-label="Delete share link"
                    >
                      🗑️ Delete
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="dialog-footer">
          <button className="close-footer-button" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default ShareDialog;
