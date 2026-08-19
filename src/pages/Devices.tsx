import React, { useState } from 'react';
import { Button, Space, Tabs } from '@arco-design/web-react';
import { IconPlus, IconList, IconFolder } from '@arco-design/web-react/icon';
import { DeviceList } from '../components/Devices/DeviceList';
import { DeviceFiles } from '../components/Devices/DeviceFiles';
import { BindDeviceDialog } from '../components/Devices/BindDeviceDialog';
import { useDevicesStore } from '../store/devicesStore';
import './Devices.css';

const TabPane = Tabs.TabPane;

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
          icon={<IconPlus />}
          onClick={() => setBindDialogVisible(true)}
        >
          Bind Device
        </Button>
      </div>

      <Tabs
        activeTab={activeTab}
        onChange={setActiveTab}
        type="card-gutter"
        className="devices-tabs"
      >
        <TabPane
          key="list"
          title={
            <span>
              <IconList style={{ marginRight: 6 }} />
              Device List
            </span>
          }
        >
          <DeviceList onDeviceSelect={handleDeviceSelect} />
        </TabPane>

        <TabPane
          key="files"
          title={
            <span>
              <IconFolder style={{ marginRight: 6 }} />
              Files {connectedDevice ? `- ${connectedDevice.name}` : ''}
            </span>
          }
          disabled={!connectedDevice}
        >
          {connectedDevice ? (
            <DeviceFiles deviceId={connectedDevice.id} />
          ) : (
            <div className="device-files-empty">
              <p>Select a connected device to view files</p>
            </div>
          )}
        </TabPane>
      </Tabs>

      <BindDeviceDialog
        visible={bindDialogVisible}
        onClose={() => setBindDialogVisible(false)}
      />
    </div>
  );
}
