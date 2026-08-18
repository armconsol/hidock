import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import ShareDialog from './ShareDialog';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn(() => Promise.resolve()),
  },
});

describe('ShareDialog', () => {
  const mockNoteId = 'test-note-id';
  const mockNoteTitle = 'Test Note Title';
  const mockOnClose = vi.fn();

  const mockShareLink = {
    id: 'share-1',
    note_id: mockNoteId,
    token: 'test-token-123456789',
    expires_at: null,
    created_at: '2024-01-01T00:00:00Z',
    last_accessed_at: null,
    access_count: 0,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([]);
  });

  it('renders dialog when open', async () => {
    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    await waitFor(() => {
      expect(screen.getByText('Share Note')).toBeInTheDocument();
      expect(screen.getByText(mockNoteTitle)).toBeInTheDocument();
    });
  });

  it('does not render when closed', () => {
    const { container } = render(
      <ShareDialog
        isOpen={false}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    expect(container.firstChild).toBeNull();
  });

  it('loads share links on open', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([mockShareLink]);

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('list_share_links', { noteId: mockNoteId });
    });
  });

  it('displays share links', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([mockShareLink]);

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    await waitFor(() => {
      expect(screen.getByText(/test-token-123456789/i, { exact: false })).toBeInTheDocument();
    });
  });

  it('shows "no links" message when no share links exist', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([]);

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    await waitFor(() => {
      expect(screen.getByText(/No share links created yet/i)).toBeInTheDocument();
    });
  });

  it('generates new share link', async () => {
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce([]) // initial list
      .mockResolvedValueOnce(mockShareLink) // create response
      .mockResolvedValueOnce([mockShareLink]); // updated list

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    const generateButton = await screen.findByText('Generate Link');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith(
        'create_share_link',
        expect.objectContaining({
          noteId: mockNoteId,
          expiresInDays: 7,
        })
      );
    });
  });

  it('copies share link to clipboard', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([mockShareLink]);

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    const copyButton = await screen.findByText(/Copy Link/i);
    fireEvent.click(copyButton);

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalled();
    });
  });

  it('deletes share link', async () => {
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce([mockShareLink]) // initial list
      .mockResolvedValueOnce(undefined) // delete response
      .mockResolvedValueOnce([]); // updated list

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    const deleteButton = await screen.findByText(/Delete/i);
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('delete_share_link', {
        shareId: mockShareLink.id,
      });
    });
  });

  it('closes dialog on close button click', async () => {
    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    const closeButton = screen.getAllByText('Close')[0];
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it('handles error when loading share links fails', async () => {
    const errorMessage = 'Failed to load links';
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(errorMessage);

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    await waitFor(() => {
      expect(screen.getByText(/Failed to load share links/i)).toBeInTheDocument();
    });
  });

  it('changes expiration days', async () => {
    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    const select = (await screen.findByLabelText(/Expires in:/i)) as HTMLSelectElement;
    fireEvent.change(select, { target: { value: '30' } });

    expect(select.value).toBe('30');
  });

  it('shows expired status for expired links', async () => {
    const expiredLink = {
      ...mockShareLink,
      expires_at: '2020-01-01T00:00:00Z', // Past date
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([expiredLink]);

    render(
      <ShareDialog
        isOpen={true}
        onClose={mockOnClose}
        noteId={mockNoteId}
        noteTitle={mockNoteTitle}
      />
    );

    await waitFor(() => {
      expect(screen.getByText(/Expired/i)).toBeInTheDocument();
    });
  });
});
