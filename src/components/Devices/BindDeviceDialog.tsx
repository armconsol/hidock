import { useState } from 'react';
import { Modal, Form, Input, message } from 'antd';
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
      const values = await form.validateFields();
      setLoading(true);

      await bindDevice({
        id: values.deviceId,
        name: values.deviceName,
      });

      message.success('Device bound successfully');
      form.resetFields();
      onClose();
    } catch (error) {
      if (error instanceof Error) {
        message.error(`Failed to bind device: ${error.message}`);
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
      open={visible}
      onOk={handleSubmit}
      onCancel={handleCancel}
      confirmLoading={loading}
      okText="Bind Device"
      cancelText="Cancel"
    >
      <Form form={form} layout="vertical">
        <Form.Item
          label="Device ID"
          name="deviceId"
          rules={[
            { required: true, message: 'Please enter device ID' },
            { min: 3, message: 'Device ID must be at least 3 characters' },
          ]}
        >
          <Input placeholder="Enter device ID (e.g., HIDOC-12345)" />
        </Form.Item>
        <Form.Item
          label="Device Name"
          name="deviceName"
          rules={[
            { required: true, message: 'Please enter device name' },
            { min: 2, message: 'Device name must be at least 2 characters' },
          ]}
        >
          <Input placeholder="Enter a friendly name for this device" />
        </Form.Item>
      </Form>
    </Modal>
  );
}
