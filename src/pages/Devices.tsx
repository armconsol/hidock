import { useState } from 'react';
import { Button, Tabs } from 'antd';
import { PlusOutlined, UnorderedListOutlined, FolderOutlined } from '@ant-design/icons';
import { DeviceList } from '../components/Devices/DeviceList';
import { DeviceFiles } from '../components/Devices/DeviceFiles';
import { BindDeviceDialog } from '../components/Devices/BindDeviceDialog';
import { useDevicesStore } from '../store/devicesStore';
import './Devices.css';

export function DevicesPage() {
  const [bindDialogVisible, setBindDialogVisible] = useState(false);
  const [selectedDeviceId, setSelectedDeviceId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<string>('list');
  const { devices } = useDevicesStore();

  const handleDeviceSelect = (deviceId: string) => {
    setSelectedDeviceId(deviceId);
    setActiveTab('files');
  };

  const connectedDevice = devices.find(
    (d) => d.id === selectedDeviceId && d.status === 'connected'
  );

  return (
    <div className="devices-page">
      <div className="devices-header">
        <div className="devices-title-section">
          <h1>Devices</h1>
          <p className="devices-subtitle">Manage your connected HiDoc P1 devices</p>
        </div>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setBindDialogVisible(true)}
        >
          Bind Device
        </Button>
      </div>

      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        type="card"
        className="devices-tabs"
        items={[
          {
            key: 'list',
            label: (
              <span>
                <UnorderedListOutlined style={{ marginRight: 6 }} />
                Device List
              </span>
            ),
            children: <DeviceList onDeviceSelect={handleDeviceSelect} />,
          },
          {
            key: 'files',
            label: (
              <span>
                <FolderOutlined style={{ marginRight: 6 }} />
                Files {connectedDevice ? `- ${connectedDevice.name}` : ''}
              </span>
            ),
            disabled: !connectedDevice,
            children: connectedDevice ? (
              <DeviceFiles deviceId={connectedDevice.id} />
            ) : (
              <div className="device-files-empty">
                <p>Select a connected device to view files</p>
              </div>
            ),
          },
        ]}
      />

      <BindDeviceDialog
        visible={bindDialogVisible}
        onClose={() => setBindDialogVisible(false)}
      />
    </div>
  );
}
