import { Button, Space, Tooltip } from 'antd';
import {
  FileOutlined,
  CheckCircleOutlined,
  CalendarOutlined,
} from '@ant-design/icons';
import './WhisperActions.css';

interface WhisperActionsProps {
  whisperId: string;
  onConvert?: (targetType: 'note' | 'todo' | 'calendar') => void;
  disabled?: boolean;
}

export function WhisperActions({
  whisperId,
  onConvert,
  disabled = false,
}: WhisperActionsProps) {
  const handleConvert = (targetType: 'note' | 'todo' | 'calendar') => {
    onConvert?.(targetType);
  };

  return (
    <div className="whisper-actions" data-testid={`whisper-actions-${whisperId}`}>
      <Space size="small">
        <Tooltip title="Convert to Note">
          <Button
            type="default"
            size="small"
            icon={<FileOutlined />}
            onClick={() => handleConvert('note')}
            disabled={disabled}
            data-testid="convert-to-note"
          >
            Note
          </Button>
        </Tooltip>

        <Tooltip title="Convert to To-Do">
          <Button
            type="default"
            size="small"
            icon={<CheckCircleOutlined />}
            onClick={() => handleConvert('todo')}
            disabled={disabled}
            data-testid="convert-to-todo"
          >
            To-Do
          </Button>
        </Tooltip>

        <Tooltip title="Extract Calendar Event">
          <Button
            type="default"
            size="small"
            icon={<CalendarOutlined />}
            onClick={() => handleConvert('calendar')}
            disabled={disabled}
            data-testid="convert-to-calendar"
          >
            Calendar
          </Button>
        </Tooltip>
      </Space>
    </div>
  );
}
