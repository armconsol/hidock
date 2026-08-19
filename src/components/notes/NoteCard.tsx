import { Card, Typography, Space, Tag } from '@arco-design/web-react';
import { IconPushpin, IconClockCircle } from '@arco-design/web-react/icon';
import type { Note } from '../../types/notes';
import './NoteCard.css';

const { Text } = Typography;

interface NoteCardProps {
  note: Note;
  isSelected?: boolean;
  onClick?: () => void;
}

export function NoteCard({ note, isSelected, onClick }: NoteCardProps) {
  const formatDuration = (seconds?: number): string => {
    if (!seconds) return '';
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (date: Date): string => {
    const d = new Date(date);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) {
      return 'Today';
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return `${days} days ago`;
    } else {
      return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
  };

  return (
    <Card
      className={`note-card ${isSelected ? 'note-card-selected' : ''}`}
      hoverable
      onClick={onClick}
      bordered={false}
    >
      <Space direction="vertical" size={4} style={{ width: '100%' }}>
        <div className="note-card-header">
          <Space>
            {note.isPinned && (
              <IconPushpin style={{ fontSize: 14, color: 'var(--color-text-3)' }} />
            )}
            <Text ellipsis className="note-card-title">
              {note.title}
            </Text>
          </Space>
        </div>
        <div className="note-card-meta">
          <Space size={12}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {formatDate(note.updatedAt)}
            </Text>
            {note.duration && (
              <Space size={4}>
                <IconClockCircle style={{ fontSize: 12 }} />
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {formatDuration(note.duration)}
                </Text>
              </Space>
            )}
          </Space>
        </div>
        {note.tags && note.tags.length > 0 && (
          <Space size={4} wrap>
            {note.tags.slice(0, 3).map((tag) => (
              <Tag key={tag} size="small" color="arcoblue">
                {tag}
              </Tag>
            ))}
            {note.tags.length > 3 && (
              <Text type="secondary" style={{ fontSize: 12 }}>
                +{note.tags.length - 3}
              </Text>
            )}
          </Space>
        )}
      </Space>
    </Card>
  );
}
