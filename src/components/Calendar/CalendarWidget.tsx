import { Calendar } from '@arco-design/web-react';
import { useEffect, useState } from 'react';
import './CalendarWidget.css';

interface CalendarEvent {
  id: string;
  title: string;
  start_time: string;
  end_time: string;
  source: 'google_calendar' | 'hinotes';
  meeting_url?: string;
}

export function CalendarWidget() {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [todayEvents, setTodayEvents] = useState<CalendarEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTodayEvents();
  }, []);

  const loadTodayEvents = async () => {
    try {
      setLoading(true);
      setError(null);

      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const tomorrow = new Date(today);
      tomorrow.setDate(tomorrow.getDate() + 1);

      // TODO: Replace with actual API call to Tauri backend
      // const response = await invoke('get_calendar_events', {
      //   startDate: today.toISOString(),
      //   endDate: tomorrow.toISOString()
      // });

      // Mock data for now
      const mockEvents: CalendarEvent[] = [
        {
          id: '1',
          title: 'Team Standup',
          start_time: new Date(today.setHours(9, 0)).toISOString(),
          end_time: new Date(today.setHours(9, 30)).toISOString(),
          source: 'google_calendar',
          meeting_url: 'https://meet.google.com/abc-def-ghi',
        },
        {
          id: '2',
          title: 'Project Review',
          start_time: new Date(today.setHours(14, 0)).toISOString(),
          end_time: new Date(today.setHours(15, 0)).toISOString(),
          source: 'google_calendar',
        },
        {
          id: '3',
          title: 'Quick Note Review',
          start_time: new Date(today.setHours(16, 30)).toISOString(),
          end_time: new Date(today.setHours(17, 0)).toISOString(),
          source: 'hinotes',
        },
      ];

      setEvents(mockEvents);
      setTodayEvents(mockEvents);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load calendar events');
      console.error('Error loading calendar events:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  };

  const isEventNow = (event: CalendarEvent) => {
    const now = new Date();
    const start = new Date(event.start_time);
    const end = new Date(event.end_time);
    return now >= start && now <= end;
  };

  const getEventSource = (source: string) => {
    return source === 'google_calendar' ? 'Google Calendar' : 'HiNotes';
  };

  if (loading) {
    return (
      <div className="calendar-widget">
        <div className="calendar-widget-header">
          <h3>Today's Schedule</h3>
        </div>
        <div className="calendar-widget-loading">Loading events...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="calendar-widget">
        <div className="calendar-widget-header">
          <h3>Today's Schedule</h3>
        </div>
        <div className="calendar-widget-error">{error}</div>
      </div>
    );
  }

  return (
    <div className="calendar-widget">
      <div className="calendar-widget-header">
        <h3>Today's Schedule</h3>
        <span className="event-count">{todayEvents.length} events</span>
      </div>

      <div className="calendar-container">
        <Calendar />
      </div>

      <div className="events-list">
        <h4>Today's Events</h4>
        {todayEvents.length === 0 ? (
          <div className="no-events">No events scheduled for today</div>
        ) : (
          <ul className="events-list-items">
            {todayEvents.map((event) => (
              <li
                key={event.id}
                className={`event-item ${isEventNow(event) ? 'event-active' : ''}`}
              >
                <div className="event-time">
                  {formatTime(event.start_time)} - {formatTime(event.end_time)}
                </div>
                <div className="event-details">
                  <div className="event-title">{event.title}</div>
                  <div className="event-source">{getEventSource(event.source)}</div>
                </div>
                {event.meeting_url && (
                  <a
                    href={event.meeting_url}
                    className="event-join-link"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Join
                  </a>
                )}
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
