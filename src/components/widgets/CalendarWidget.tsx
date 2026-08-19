import { useState, useEffect } from 'react';
import { Calendar, Card, Empty, Typography, Space, Tag } from '@arco-design/web-react';
import { invoke } from '@tauri-apps/api/core';
import './CalendarWidget.css';

const { Title, Text } = Typography;

interface CalendarEvent {
  id: string;
  title: string;
  start_time: string;
  end_time: string;
  source: 'google_calendar' | 'hinotes';
  meeting_url: string | null;
  created_at: string;
  synced_at: string | null;
}

interface CalendarWidgetProps {
  onEventClick?: (event: CalendarEvent) => void;
}

export function CalendarWidget({ onEventClick }: CalendarWidgetProps) {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [selectedDate, setSelectedDate] = useState<Date>(new Date());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadEvents();
  }, [selectedDate]);

  const loadEvents = async () => {
    try {
      setLoading(true);
      setError(null);

      const startOfDay = new Date(selectedDate);
      startOfDay.setHours(0, 0, 0, 0);

      const endOfDay = new Date(selectedDate);
      endOfDay.setHours(23, 59, 59, 999);

      const fetchedEvents = await invoke<CalendarEvent[]>('get_calendar_events', {
        startTime: startOfDay.toISOString(),
        endTime: endOfDay.toISOString(),
      });

      setEvents(fetchedEvents);
    } catch (err) {
      console.error('Failed to load calendar events:', err);
      setError('Failed to load events. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleDateChange = (date: any) => {
    // dayjs object from Arco Design Calendar, convert to Date
    const jsDate = date instanceof Date ? date : new Date(date.format('YYYY-MM-DD'));
    setSelectedDate(jsDate);
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  };

  const todayEvents = events.filter((event) => {
    const eventDate = new Date(event.start_time);
    return eventDate.toDateString() === selectedDate.toDateString();
  });

  return (
    <Card
      className="calendar-widget"
      title="Calendar"
      bordered={false}
    >
      <Space direction="vertical" size="medium" style={{ width: '100%' }}>
        <Calendar
          panel
          value={selectedDate}
          onChange={handleDateChange}
          style={{ width: '100%' }}
        />

        <div className="calendar-events">
          <Title heading={6} style={{ marginBottom: 16 }}>
            Today's Events
          </Title>

          {loading && <Text>Loading events...</Text>}

          {error && (
            <Text type="error">{error}</Text>
          )}

          {!loading && !error && todayEvents.length === 0 && (
            <Empty description="No events scheduled for today" />
          )}

          {!loading && !error && todayEvents.length > 0 && (
            <Space direction="vertical" size="small" style={{ width: '100%' }}>
              {todayEvents.map((event) => (
                <Card
                  key={event.id}
                  className="event-card"
                  size="small"
                  hoverable
                  onClick={() => onEventClick?.(event)}
                >
                  <Space direction="vertical" size="mini" style={{ width: '100%' }}>
                    <Space>
                      <Text style={{ fontWeight: 600 }}>{event.title}</Text>
                      <Tag color={event.source === 'google_calendar' ? 'blue' : 'green'}>
                        {event.source === 'google_calendar' ? 'Google' : 'HiNotes'}
                      </Tag>
                    </Space>

                    <Text type="secondary">
                      {formatTime(event.start_time)} - {formatTime(event.end_time)}
                    </Text>

                    {event.meeting_url && (
                      <a
                        href={event.meeting_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        onClick={(e) => e.stopPropagation()}
                      >
                        Join Meeting
                      </a>
                    )}
                  </Space>
                </Card>
              ))}
            </Space>
          )}
        </div>
      </Space>
    </Card>
  );
}
