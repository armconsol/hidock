import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { CalendarWidget } from './CalendarWidget';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('CalendarWidget', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render calendar widget', () => {
    render(<CalendarWidget />);
    expect(screen.getByText(/calendar/i)).toBeInTheDocument();
  });

  it('should display today\'s events', async () => {
    const { invoke } = await import('@tauri-apps/api/core');

    // Use a fixed mid-day time so "+2 hours" and "+3 hours" offsets below
    // never roll over into the next calendar day (which would make
    // CalendarWidget's todayEvents filter exclude them, causing this test
    // to fail flakily depending on the time of day it happens to run).
    const todayNoon = new Date();
    todayNoon.setHours(12, 0, 0, 0);

    // Mock calendar events
    (invoke as any).mockResolvedValueOnce([
      {
        id: 'event-1',
        title: 'Team Meeting',
        start_time: todayNoon.toISOString(),
        end_time: new Date(todayNoon.getTime() + 3600000).toISOString(),
        source: 'google_calendar',
        meeting_url: 'https://meet.google.com/abc-def-ghi',
      },
      {
        id: 'event-2',
        title: 'Code Review',
        start_time: new Date(todayNoon.getTime() + 7200000).toISOString(),
        end_time: new Date(todayNoon.getTime() + 10800000).toISOString(),
        source: 'hinotes',
        meeting_url: null,
      },
    ]);

    render(<CalendarWidget />);

    await waitFor(() => {
      expect(screen.getByText('Team Meeting')).toBeInTheDocument();
      expect(screen.getByText('Code Review')).toBeInTheDocument();
    });
  });

  it('should show empty state when no events', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    (invoke as any).mockResolvedValueOnce([]);

    render(<CalendarWidget />);

    await waitFor(() => {
      expect(screen.getByText(/no events/i)).toBeInTheDocument();
    });
  });

  it('should display meeting links for events with URLs', async () => {
    const { invoke } = await import('@tauri-apps/api/core');

    // Fixed mid-day time so the +1 hour offset never rolls into the next
    // calendar day (see comment in the "today's events" test above).
    const todayNoon = new Date();
    todayNoon.setHours(12, 0, 0, 0);

    (invoke as any).mockResolvedValueOnce([
      {
        id: 'event-1',
        title: 'Team Meeting',
        start_time: todayNoon.toISOString(),
        end_time: new Date(todayNoon.getTime() + 3600000).toISOString(),
        source: 'google_calendar',
        meeting_url: 'https://meet.google.com/abc-def-ghi',
      },
    ]);

    render(<CalendarWidget />);

    await waitFor(() => {
      const link = screen.getByRole('link', { name: /join/i });
      expect(link).toHaveAttribute('href', 'https://meet.google.com/abc-def-ghi');
    });
  });

  it('should handle API errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    (invoke as any).mockRejectedValueOnce(new Error('Failed to fetch events'));

    render(<CalendarWidget />);

    await waitFor(() => {
      expect(screen.getByText(/failed to load events/i)).toBeInTheDocument();
    });
  });
});
