import { List, Card, Typography, Space, Empty, Spin, Button } from 'antd';
import { ClockCircleOutlined, DeleteOutlined } from '@ant-design/icons';
import type { Whisper } from '../../types/whispers';
import { WhisperActions } from './WhisperActions';
import './WhisperList.css';

const { Text, Paragraph } = Typography;

interface WhisperListProps {
  whispers: Whisper[];
  loading?: boolean;
  onDelete?: (whisperId: string) => void;
  onConvert?: (whisperId: string, targetType: 'note' | 'todo' | 'calendar') => void;
  onPlayAudio?: (audioUrl: string) => void;
}

export function WhisperList({
  whispers,
  loading = false,
  onDelete,
  onConvert,
  onPlayAudio,
}: WhisperListProps) {
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatTimestamp = (date: Date): string => {
    const d = new Date(date);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    const minutes = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (minutes < 1) {
      return 'Just now';
    } else if (minutes < 60) {
      return `${minutes}m ago`;
    } else if (hours < 24) {
      return `${hours}h ago`;
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return `${days}d ago`;
    } else {
      return d.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    }
  };

  if (loading) {
    return (
      <div className="whisper-list-loading" data-testid="whisper-list-loading">
        <Spin size="large" />
        <Text type="secondary" style={{ marginTop: 16 }}>
          Loading whispers...
        </Text>
      </div>
    );
  }

  if (whispers.length === 0) {
    return (
      <Empty
        description="No whispers yet"
        style={{ marginTop: 100 }}
        data-testid="whisper-list-empty"
      />
    );
  }

  return (
    <List
      className="whisper-list"
      dataSource={whispers}
      data-testid="whisper-list"
      renderItem={(whisper, index) => (
        <Card
          key={whisper.id}
          className="whisper-card"
          hoverable
          bordered={false}
          data-testid={`whisper-card-${index}`}
        >
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            {/* Header with timestamp and duration */}
            <div className="whisper-card-header">
              <Space size="small">
                <ClockCircleOutlined style={{ fontSize: 14, color: 'rgba(0, 0, 0, 0.45)' }} />
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {formatTimestamp(whisper.createdAt)}
                </Text>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  • {formatDuration(whisper.duration)}
                </Text>
              </Space>
              <Button
                type="text"
                size="small"
                icon={<DeleteOutlined />}
                onClick={() => onDelete?.(whisper.id)}
                data-testid={`delete-button-${index}`}
                danger
              />
            </div>

            {/* Transcribed content */}
            <Paragraph
              className="whisper-content"
              ellipsis={{
                rows: 3,
                expandable: true,
              }}
              style={{ margin: 0 }}
            >
              {whisper.content}
            </Paragraph>

            {/* Audio playback */}
            <div className="whisper-audio">
              <Button
                type="default"
                size="small"
                onClick={() => onPlayAudio?.(whisper.audioUrl)}
                data-testid={`play-button-${index}`}
              >
                Play Audio
              </Button>
            </div>

            {/* Convert actions */}
            <WhisperActions
              whisperId={whisper.id}
              onConvert={(targetType) => onConvert?.(whisper.id, targetType)}
            />
          </Space>
        </Card>
      )}
    />
  );
}
