import React, { useState } from 'react';
import { Modal, Form, Input, Message } from '@arco-design/web-react';
import { useDevicesStore } from '../../store/devicesStore';

interface BindDeviceDialogProps {
  visible: boolean;
  onClose: () => void;
}

export function BindDeviceDialog({ visible, onClose }: BindDeviceDialogProps) {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const bindDevice = useDevicesStore((state) => state.bindDevice);

  const handleSubmit = async () => {
    try {
      const values = await form.validate();
      setLoading(true);

      await bindDevice({
        id: values.deviceId,
        name: values.deviceName,
      });

      Message.success('Device bound successfully');
      form.resetFields();
      onClose();
    } catch (error) {
      if (error instanceof Error) {
        Message.error(`Failed to bind device: ${error.message}`);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    form.resetFields();
    onClose();
  };

  return (
    <Modal
      title="Bind New Device"
      visible={visible}
      onOk={handleSubmit}
      onCancel={handleCancel}
      confirmLoading={loading}
      okText="Bind Device"
      cancelText="Cancel"
    >
      <Form form={form} layout="vertical">
        <Form.Item
          label="Device ID"
          field="deviceId"
          rules={[
            { required: true, message: 'Please enter device ID' },
            { minLength: 3, message: 'Device ID must be at least 3 characters' },
          ]}
        >
          <Input placeholder="Enter device ID (e.g., HIDOC-12345)" />
        </Form.Item>
        <Form.Item
          label="Device Name"
          field="deviceName"
          rules={[
            { required: true, message: 'Please enter device name' },
            { minLength: 2, message: 'Device name must be at least 2 characters' },
          ]}
        >
          <Input placeholder="Enter a friendly name for this device" />
        </Form.Item>
      </Form>
    </Modal>
  );
}
