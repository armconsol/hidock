import { useEffect } from 'react';
import { Card, Button, Badge, Space, Empty, Spin, message, Popconfirm } from 'antd';
import { WifiOutlined, PoweroffOutlined, SyncOutlined, DeleteOutlined } from '@ant-design/icons';
import { useDevicesStore } from '../../store/devicesStore';
import type { Device } from '../../types/devices';
import './DeviceList.css';

interface DeviceListProps {
  onDeviceSelect?: (deviceId: string) => void;
}

export function DeviceList({ onDeviceSelect }: DeviceListProps) {
  const { devices, loading, error, fetchDevices, unbindDevice } = useDevicesStore();

  useEffect(() => {
    fetchDevices();
  }, [fetchDevices]);

  const handleUnbind = async (deviceId: string) => {
    try {
      await unbindDevice(deviceId);
      message.success('Device unbound successfully');
    } catch (error) {
      message.error('Failed to unbind device');
    }
  };

  const formatLastSync = (lastSync: string | null) => {
    if (!lastSync) return 'Never';

    const syncDate = new Date(lastSync);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - syncDate.getTime());
    const diffMinutes = Math.floor(diffTime / (1000 * 60));
    const diffHours = Math.floor(diffTime / (1000 * 60 * 60));
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

    if (diffMinutes < 1) {
      return 'Just now';
    } else if (diffMinutes < 60) {
      return `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`;
    } else if (diffHours < 24) {
      return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    } else {
      return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    }
  };

  const getStatusBadge = (status: Device['status']) => {
    if (status === 'connected') {
      return <Badge status="success" text="Connected" />;
    }
    return <Badge status="default" text="Disconnected" />;
  };

  const getStatusIcon = (status: Device['status']) => {
    if (status === 'connected') {
      return <WifiOutlined style={{ color: '#52c41a', fontSize: 24 }} />;
    }
    return <PoweroffOutlined style={{ color: '#8c8c8c', fontSize: 24 }} />;
  };

  if (loading && devices.length === 0) {
    return (
      <div className="device-list-loading" data-testid="loading-spinner">
        <Spin size="large" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="device-list-error">
        <Empty description={`Error loading devices: ${error}`} />
      </div>
    );
  }

  if (devices.length === 0) {
    return (
      <div className="device-list-empty">
        <Empty
          description="No devices bound yet. Click 'Bind Device' to add your first HiDoc P1 device."
          image={<WifiOutlined style={{ fontSize: 64, color: '#d9d9d9' }} />}
        />
      </div>
    );
  }

  return (
    <div className="device-list-container">
      <Space direction="vertical" size="medium" className="device-list">
        {devices.map((device) => (
          <Card key={device.id} className="device-card" bordered>
            <div
              className="device-card-content"
              onClick={() => {
                if (device.status === 'connected' && onDeviceSelect) {
                  onDeviceSelect(device.id);
                }
              }}
              style={{ cursor: device.status === 'connected' ? 'pointer' : 'default' }}
            >
              <div className="device-icon">{getStatusIcon(device.status)}</div>
              <div className="device-info">
                <div className="device-header">
                  <h3 className="device-name">{device.name}</h3>
                  {getStatusBadge(device.status)}
                </div>
                <div className="device-meta">
                  <span className="device-id">ID: {device.id}</span>
                  <span className="device-sync">
                    <SyncOutlined className="sync-icon" />
                    Last sync: {formatLastSync(device.last_sync)}
                  </span>
                </div>
                {device.status === 'connected' && onDeviceSelect && (
                  <p className="device-hint">Click to view files</p>
                )}
              </div>
              <div className="device-actions" onClick={(e) => e.stopPropagation()}>
                <Popconfirm
                  title="Unbind Device"
                  description={`Are you sure you want to unbind ${device.name}? This will remove all device settings.`}
                  onConfirm={() => handleUnbind(device.id)}
                  okText="Unbind"
                  cancelText="Cancel"
                >
                  <Button
                    type="text"
                    danger
                    icon={<DeleteOutlined />}
                    className="unbind-button"
                  >
                    Unbind
                  </Button>
                </Popconfirm>
              </div>
            </div>
          </Card>
        ))}
      </Space>
    </div>
  );
}
