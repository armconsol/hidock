import { Button, Space, Tooltip } from '@arco-design/web-react';
import {
  IconFile,
  IconCheckCircle,
  IconCalendar,
} from '@arco-design/web-react/icon';
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
        <Tooltip content="Convert to Note">
          <Button
            type="outline"
            size="small"
            icon={<IconFile />}
            onClick={() => handleConvert('note')}
            disabled={disabled}
            data-testid="convert-to-note"
          >
            Note
          </Button>
        </Tooltip>

        <Tooltip content="Convert to To-Do">
          <Button
            type="outline"
            size="small"
            icon={<IconCheckCircle />}
            onClick={() => handleConvert('todo')}
            disabled={disabled}
            data-testid="convert-to-todo"
          >
            To-Do
          </Button>
        </Tooltip>

        <Tooltip content="Extract Calendar Event">
          <Button
            type="outline"
            size="small"
            icon={<IconCalendar />}
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
