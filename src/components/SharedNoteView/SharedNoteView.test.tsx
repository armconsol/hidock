import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import SharedNoteView from './SharedNoteView';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('SharedNoteView', () => {
  const mockToken = 'test-token-123';
  const mockNote = {
    note_id: 'note-1',
    title: 'Shared Note Title',
    content: 'This is the content of the shared note.',
    created_at: '2024-01-01T10:00:00Z',
    updated_at: '2024-01-02T15:30:00Z',
  };

  const renderWithRouter = (token: string) => {
    return render(
      <MemoryRouter initialEntries={[`/shared/${token}`]}>
        <Routes>
          <Route path="/shared/:token" element={<SharedNoteView />} />
        </Routes>
      </MemoryRouter>
    );
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows loading state initially', () => {
    (invoke as ReturnType<typeof vi.fn>).mockImplementation(
      () => new Promise(() => {}) // Never resolves
    );

    renderWithRouter(mockToken);

    expect(screen.getByText(/Loading shared note/i)).toBeInTheDocument();
  });

  it('loads and displays shared note', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockNote);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('get_shared_note', { token: mockToken });
    });

    expect(screen.getByText(mockNote.title)).toBeInTheDocument();
    expect(screen.getByText(mockNote.content)).toBeInTheDocument();
  });

  it('displays note metadata', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockNote);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/Created:/i)).toBeInTheDocument();
      expect(screen.getByText(/Updated:/i)).toBeInTheDocument();
    });
  });

  it('handles note without content', async () => {
    const noteWithoutContent = {
      ...mockNote,
      content: null,
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(noteWithoutContent);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/This note has no content/i)).toBeInTheDocument();
    });
  });

  it('shows error when note loading fails', async () => {
    const errorMessage = 'Share link not found or expired';
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(errorMessage);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/Unable to Load Note/i)).toBeInTheDocument();
      expect(screen.getByText(new RegExp(errorMessage, 'i'))).toBeInTheDocument();
    });
  });

  it('shows error for invalid token', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValue('Invalid token');

    renderWithRouter('invalid-token');

    await waitFor(() => {
      expect(screen.getByText(/Unable to Load Note/i)).toBeInTheDocument();
    });
  });

  it('shows generic error message on unknown error', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(null);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/Unable to Load Note/i)).toBeInTheDocument();
      expect(
        screen.getByText(/Failed to load note\. The link may have expired/i)
      ).toBeInTheDocument();
    });
  });

  it('displays "View in App" button', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockNote);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/View in App/i)).toBeInTheDocument();
    });
  });

  it('displays "Go to Home" button on error', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValue('Error');

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/Go to Home/i)).toBeInTheDocument();
    });
  });

  it('displays footer with information', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockNote);

    renderWithRouter(mockToken);

    await waitFor(() => {
      expect(screen.getByText(/This note was shared with you via HiNotes/i)).toBeInTheDocument();
      expect(screen.getByText(/Learn more about HiNotes/i)).toBeInTheDocument();
    });
  });

  it('formats dates correctly', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockNote);

    renderWithRouter(mockToken);

    await waitFor(() => {
      // Check that dates are present (exact format may vary by locale)
      const createdText = screen.getByText(/Created:/i).parentElement?.textContent;
      const updatedText = screen.getByText(/Updated:/i).parentElement?.textContent;

      expect(createdText).toContain('Created:');
      expect(updatedText).toContain('Updated:');
    });
  });
});
