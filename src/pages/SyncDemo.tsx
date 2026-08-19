import { Card, Space, Button, Divider, Typography, Switch } from '@arco-design/web-react';
import { SyncIndicator, SyncButton } from '../components/Sync';
import { useSyncStore } from '../store/syncStore';
import type { SyncStatus, SyncOperationType } from '../types/sync';

const { Title, Paragraph, Text } = Typography;

export function SyncDemo() {
  const {
    status,
    isOnline,
    lastSyncTime,
    pendingOperations,
    error,
    setStatus,
    setOnline,
    addPendingOperation,
    clearPendingOperations,
    triggerSync,
  } = useSyncStore();

  const handleAddOperation = (
    type: SyncOperationType,
    entityType: 'note' | 'todo' | 'folder' | 'template'
  ) => {
    const entityId = `${entityType}-${Date.now()}`;
    addPendingOperation(type, entityType, entityId, {
      title: `Demo ${type} ${entityType}`,
    });
  };

  const handleStatusChange = (newStatus: SyncStatus) => {
    setStatus(newStatus);
  };

  return (
    <div style={{ padding: 24 }}>
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <div>
          <Title heading={2}>Sync Status UI Demo</Title>
          <Paragraph>
            This page demonstrates the sync status indicator and manual sync button components.
          </Paragraph>
        </div>

        <Card title="Sync Components">
          <Space size="large">
            <div>
              <Text bold>Status Indicator:</Text>
              <div style={{ marginTop: 8 }}>
                <SyncIndicator />
              </div>
            </div>

            <Divider type="vertical" style={{ height: 60 }} />

            <div>
              <Text bold>Manual Sync Button:</Text>
              <div style={{ marginTop: 8 }}>
                <SyncButton />
              </div>
            </div>
          </Space>
        </Card>

        <Card title="Current State">
          <Space direction="vertical" size="medium" style={{ width: '100%' }}>
            <div>
              <Text bold>Status: </Text>
              <Text type={status === 'failed' ? 'error' : status === 'synced' ? 'success' : undefined}>
                {status}
              </Text>
            </div>

            <div>
              <Text bold>Online: </Text>
              <Switch
                checked={isOnline}
                onChange={setOnline}
              />
            </div>

            <div>
              <Text bold>Pending Operations: </Text>
              <Text>{pendingOperations.length}</Text>
            </div>

            <div>
              <Text bold>Last Sync: </Text>
              <Text>
                {lastSyncTime
                  ? new Date(lastSyncTime).toLocaleString()
                  : 'Never'}
              </Text>
            </div>

            {error && (
              <div>
                <Text bold>Error: </Text>
                <Text type="error">{error}</Text>
              </div>
            )}
          </Space>
        </Card>

        <Card title="Test Controls">
          <Space direction="vertical" size="medium" style={{ width: '100%' }}>
            <div>
              <Text bold>Change Status:</Text>
              <div style={{ marginTop: 8 }}>
                <Space>
                  <Button
                    size="small"
                    type={status === 'synced' ? 'primary' : 'default'}
                    onClick={() => handleStatusChange('synced')}
                  >
                    Synced
                  </Button>
                  <Button
                    size="small"
                    type={status === 'syncing' ? 'primary' : 'default'}
                    onClick={() => handleStatusChange('syncing')}
                  >
                    Syncing
                  </Button>
                  <Button
                    size="small"
                    type={status === 'offline' ? 'primary' : 'default'}
                    onClick={() => handleStatusChange('offline')}
                  >
                    Offline
                  </Button>
                  <Button
                    size="small"
                    type={status === 'failed' ? 'primary' : 'default'}
                    onClick={() => handleStatusChange('failed')}
                  >
                    Failed
                  </Button>
                </Space>
              </div>
            </div>

            <Divider />

            <div>
              <Text bold>Add Pending Operations:</Text>
              <div style={{ marginTop: 8 }}>
                <Space wrap>
                  <Button
                    size="small"
                    onClick={() => handleAddOperation('create', 'note')}
                  >
                    Create Note
                  </Button>
                  <Button
                    size="small"
                    onClick={() => handleAddOperation('update', 'note')}
                  >
                    Update Note
                  </Button>
                  <Button
                    size="small"
                    onClick={() => handleAddOperation('delete', 'note')}
                  >
                    Delete Note
                  </Button>
                  <Button
                    size="small"
                    onClick={() => handleAddOperation('create', 'todo')}
                  >
                    Create Todo
                  </Button>
                  <Button
                    size="small"
                    onClick={() => handleAddOperation('update', 'folder')}
                  >
                    Update Folder
                  </Button>
                </Space>
              </div>
            </div>

            <Divider />

            <div>
              <Text bold>Actions:</Text>
              <div style={{ marginTop: 8 }}>
                <Space>
                  <Button
                    size="small"
                    type="primary"
                    onClick={triggerSync}
                    disabled={!isOnline}
                  >
                    Trigger Sync
                  </Button>
                  <Button
                    size="small"
                    status="danger"
                    onClick={clearPendingOperations}
                  >
                    Clear Pending
                  </Button>
                </Space>
              </div>
            </div>
          </Space>
        </Card>

        <Card title="Pending Operations List">
          {pendingOperations.length === 0 ? (
            <Paragraph type="secondary">No pending operations</Paragraph>
          ) : (
            <Space direction="vertical" size="small" style={{ width: '100%' }}>
              {pendingOperations.map((op, index) => (
                <Card key={op.id} size="small" bordered>
                  <Space direction="vertical" size={0}>
                    <Text bold>
                      #{index + 1}: {op.type} {op.entityType}
                    </Text>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      ID: {op.entityId}
                    </Text>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      Time: {new Date(op.timestamp).toLocaleString()}
                    </Text>
                    {op.retryCount > 0 && (
                      <Text type="warning" style={{ fontSize: 12 }}>
                        Retries: {op.retryCount}
                      </Text>
                    )}
                    {op.error && (
                      <Text type="error" style={{ fontSize: 12 }}>
                        Error: {op.error}
                      </Text>
                    )}
                  </Space>
                </Card>
              ))}
            </Space>
          )}
        </Card>
      </Space>
    </div>
  );
}
