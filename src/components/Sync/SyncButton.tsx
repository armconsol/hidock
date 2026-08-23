import { Button, message, Popover, Space, Typography, List } from 'antd';
import { SyncOutlined, ClockCircleOutlined, DeleteOutlined } from '@ant-design/icons';
import { useSyncStore } from '../../store/syncStore';
import type { SyncOperation } from '../../types/sync';
import './SyncButton.css';

const { Text, Paragraph } = Typography;

export function SyncButton() {
  const {
    status,
    pendingOperations,
    error,
    isOnline,
    triggerSync,
    clearPendingOperations,
  } = useSyncStore();

  const handleSync = async () => {
    if (!isOnline) {
      message.warning('Cannot sync while offline');
      return;
    }

    try {
      await triggerSync();
      if (status === 'synced') {
        message.success('Sync completed successfully');
      }
    } catch (err) {
      message.error('Sync failed: ' + (err instanceof Error ? err.message : 'Unknown error'));
    }
  };

  const handleClearPending = () => {
    clearPendingOperations();
    message.info('Cleared pending operations');
  };

  const formatOperationType = (type: SyncOperation['type']): string => {
    switch (type) {
      case 'create':
        return 'Create';
      case 'update':
        return 'Update';
      case 'delete':
        return 'Delete';
    }
  };

  const formatEntityType = (type: SyncOperation['entityType']): string => {
    return type.charAt(0).toUpperCase() + type.slice(1);
  };

  const formatTimestamp = (date: Date): string => {
    return new Date(date).toLocaleTimeString();
  };

  const popoverContent = (
    <div className="sync-button-popover">
      <div className="sync-button-popover-header">
        <Text strong>Pending Operations</Text>
        {pendingOperations.length > 0 && (
          <Button
            type="text"
            size="small"
            icon={<DeleteOutlined />}
            onClick={handleClearPending}
          >
            Clear All
          </Button>
        )}
      </div>

      {pendingOperations.length === 0 ? (
        <Paragraph className="sync-button-empty">
          No pending operations
        </Paragraph>
      ) : (
        <List
          size="small"
          dataSource={pendingOperations}
          renderItem={(item) => (
            <List.Item key={item.id}>
              <Space direction="vertical" size={0}>
                <Text>
                  <Text strong>{formatOperationType(item.type)}</Text>
                  {' '}
                  {formatEntityType(item.entityType)}
                </Text>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  <ClockCircleOutlined style={{ marginRight: 4 }} />
                  {formatTimestamp(item.timestamp)}
                  {item.retryCount > 0 && ` • Retry ${item.retryCount}`}
                </Text>
                {item.error && (
                  <Text type="danger" style={{ fontSize: 12 }}>
                    {item.error}
                  </Text>
                )}
              </Space>
            </List.Item>
          )}
        />
      )}

      {error && (
        <div className="sync-button-error">
          <Text type="danger">{error}</Text>
        </div>
      )}
    </div>
  );

  return (
    <Popover
      placement="bottom"
      trigger="click"
      content={popoverContent}
    >
      <Button
        type="default"
        icon={<SyncOutlined />}
        loading={status === 'syncing'}
        disabled={!isOnline}
        onClick={(e) => {
          // Prevent popover from opening when clicking sync
          e.stopPropagation();
          handleSync();
        }}
      >
        Sync
        {pendingOperations.length > 0 && ` (${pendingOperations.length})`}
      </Button>
    </Popover>
  );
}
