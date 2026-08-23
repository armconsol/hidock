import { Badge, Tooltip, Typography } from 'antd';
import { SyncOutlined, CheckOutlined, CloseOutlined, WifiOutlined } from '@ant-design/icons';
import { useSyncStore } from '../../store/syncStore';
import './SyncIndicator.css';

const { Text } = Typography;

export function SyncIndicator() {
  const { status, lastSyncTime, pendingOperations, error } = useSyncStore();

  const getStatusConfig = () => {
    switch (status) {
      case 'synced':
        return {
          icon: <CheckOutlined />,
          color: 'success',
          text: 'Synced',
          className: 'sync-indicator-synced',
        };
      case 'syncing':
        return {
          icon: <SyncOutlined spin className="sync-indicator-spinning" />,
          color: 'processing',
          text: 'Syncing...',
          className: 'sync-indicator-syncing',
        };
      case 'offline':
        return {
          icon: <WifiOutlined />,
          color: 'default',
          text: 'Offline',
          className: 'sync-indicator-offline',
        };
      case 'failed':
        return {
          icon: <CloseOutlined />,
          color: 'error',
          text: 'Failed',
          className: 'sync-indicator-failed',
        };
    }
  };

  const config = getStatusConfig();

  const formatLastSyncTime = (date: Date | null): string => {
    if (!date) return 'Never synced';

    const now = new Date();
    const diffMs = now.getTime() - new Date(date).getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  };

  const getTooltipContent = () => {
    const lines = [
      `Status: ${config.text}`,
      `Last sync: ${formatLastSyncTime(lastSyncTime)}`,
    ];

    if (pendingOperations.length > 0) {
      lines.push(`Pending: ${pendingOperations.length} operations`);
    }

    if (error) {
      lines.push(`Error: ${error}`);
    }

    return lines.join('\n');
  };

  return (
    <Tooltip title={getTooltipContent()} placement="bottom">
      <div className={`sync-indicator ${config.className}`}>
        <Badge
          count={pendingOperations.length}
          dot={pendingOperations.length > 0}
          status={config.color as any}
        >
          <div className="sync-indicator-content">
            <span className="sync-indicator-icon">{config.icon}</span>
            <Text className="sync-indicator-text">{config.text}</Text>
            {pendingOperations.length > 0 && (
              <Text className="sync-indicator-count">({pendingOperations.length})</Text>
            )}
          </div>
        </Badge>
      </div>
    </Tooltip>
  );
}
