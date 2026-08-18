import React, { useState } from 'react';
import { Button, Space } from '@arco-design/web-react';
import { IconPlus } from '@arco-design/web-react/icon';
import { DeviceList } from '../components/Devices/DeviceList';
import { BindDeviceDialog } from '../components/Devices/BindDeviceDialog';
import './Devices.css';

export function DevicesPage() {
  const [bindDialogVisible, setBindDialogVisible] = useState(false);

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

      <DeviceList />

      <BindDeviceDialog
        visible={bindDialogVisible}
        onClose={() => setBindDialogVisible(false)}
      />
    </div>
  );
}
