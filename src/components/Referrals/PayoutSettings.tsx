import { useState } from 'react';
import {
  Card,
  Button,
  Input,
  Space,
  message,
  Tag,
  Alert,
  Modal,
  Form,
} from 'antd';
import { LinkOutlined, GiftOutlined } from '@ant-design/icons';
import { PayPalConnection } from '../../types/referral';
import './PayoutSettings.css';

const FormItem = Form.Item;

interface PayoutSettingsProps {
  paypalConnection?: PayPalConnection;
  availableCash: number;
  minimumPayout: number;
  onConnectPayPal: (email: string, authCode: string) => Promise<void>;
  onDisconnectPayPal: () => Promise<void>;
  onRequestPayout: (amount: number) => Promise<void>;
}

export function PayoutSettings({
  paypalConnection,
  availableCash,
  minimumPayout,
  onConnectPayPal,
  onDisconnectPayPal,
  onRequestPayout,
}: PayoutSettingsProps) {
  const [connectModalVisible, setConnectModalVisible] = useState(false);
  const [payoutModalVisible, setPayoutModalVisible] = useState(false);
  const [loading, setLoading] = useState(false);
  const [form] = Form.useForm();
  const [payoutForm] = Form.useForm();

  const isConnected = paypalConnection?.status === 'active';
  const canRequestPayout = isConnected && availableCash >= minimumPayout;

  const handleConnectPayPal = async () => {
    try {
      const values = await form.validateFields();
      setLoading(true);
      await onConnectPayPal(values.email, values.authCode || '');
      message.success('PayPal account connected successfully!');
      setConnectModalVisible(false);
      form.resetFields();
    } catch (error) {
      if (error instanceof Error && 'errorFields' in error) {
        // Form validation error - do nothing, Ant Design will show errors
        return;
      }
      message.error('Failed to connect PayPal account');
      console.error('Error connecting PayPal:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDisconnectPayPal = async () => {
    Modal.confirm({
      title: 'Disconnect PayPal',
      content: 'Are you sure you want to disconnect your PayPal account?',
      onOk: async () => {
        setLoading(true);
        try {
          await onDisconnectPayPal();
          message.success('PayPal account disconnected');
        } catch (error) {
          message.error('Failed to disconnect PayPal account');
          console.error('Error disconnecting PayPal:', error);
        } finally {
          setLoading(false);
        }
      },
    });
  };

  const handleRequestPayout = async () => {
    try {
      const values = await payoutForm.validateFields();
      setLoading(true);
      await onRequestPayout(values.amount);
      message.success('Payout request submitted successfully!');
      setPayoutModalVisible(false);
      payoutForm.resetFields();
    } catch (error) {
      if (error instanceof Error && 'errorFields' in error) {
        // Form validation error
        return;
      }
      message.error('Failed to request payout');
      console.error('Error requesting payout:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="payout-settings-card" title="Payout Settings">
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* Available Cash */}
        <div className="available-cash-section">
          <div className="cash-display">
            <GiftOutlined style={{ fontSize: 32, color: 'var(--color-success-6)' }} />
            <div className="cash-content">
              <div className="cash-label">Available Cash</div>
              <div className="cash-amount">${availableCash.toFixed(2)}</div>
            </div>
          </div>
          {availableCash < minimumPayout && (
            <Alert
              type="info"
              message={`Minimum payout amount is $${minimumPayout.toFixed(2)}`}
              style={{ marginTop: 12 }}
            />
          )}
        </div>

        {/* PayPal Connection Status */}
        <div className="paypal-section">
          <div className="paypal-status">
            <label className="section-label">PayPal Account</label>
            {isConnected ? (
              <Space>
                <Tag color="green" icon={<LinkOutlined />}>
                  Connected
                </Tag>
                <span className="paypal-email">{paypalConnection?.email}</span>
              </Space>
            ) : (
              <Tag color="default">
                Not Connected
              </Tag>
            )}
          </div>

          <Space size="medium" style={{ marginTop: 12 }}>
            {isConnected ? (
              <>
                <Button
                  type="primary"
                  icon={<GiftOutlined />}
                  onClick={() => setPayoutModalVisible(true)}
                  disabled={!canRequestPayout}
                  loading={loading}
                >
                  Request Payout
                </Button>
                <Button
                  onClick={handleDisconnectPayPal}
                  loading={loading}
                  danger
                >
                  Disconnect PayPal
                </Button>
              </>
            ) : (
              <Button
                type="primary"
                icon={<LinkOutlined />}
                onClick={() => setConnectModalVisible(true)}
                loading={loading}
              >
                Connect PayPal
              </Button>
            )}
          </Space>
        </div>

        {/* Info Alert */}
        <Alert
          type="info"
          message="Connect your PayPal account to receive cash rewards. Minimum payout is $10.00. Payouts are processed within 5-7 business days."
        />
      </Space>

      {/* Connect PayPal Modal */}
      <Modal
        title="Connect PayPal Account"
        open={connectModalVisible}
        onCancel={() => setConnectModalVisible(false)}
        onOk={handleConnectPayPal}
        confirmLoading={loading}
      >
        <Form form={form} layout="vertical">
          <FormItem
            label="PayPal Email"
            name="email"
            rules={[
              { required: true, message: 'Please enter your PayPal email' },
              { type: 'email', message: 'Please enter a valid email address' },
            ]}
          >
            <Input placeholder="your-email@example.com" />
          </FormItem>
          <FormItem
            label="Authorization Code (Optional)"
            name="authCode"
            tooltip="Leave blank for basic connection"
          >
            <Input.Password placeholder="Optional authorization code" />
          </FormItem>
        </Form>
      </Modal>

      {/* Request Payout Modal */}
      <Modal
        title="Request Payout"
        open={payoutModalVisible}
        onCancel={() => setPayoutModalVisible(false)}
        onOk={handleRequestPayout}
        confirmLoading={loading}
      >
        <Form form={payoutForm} layout="vertical">
          <FormItem
            label="Payout Amount"
            name="amount"
            rules={[
              { required: true, message: 'Please enter payout amount' },
              {
                type: 'number',
                min: minimumPayout,
                max: availableCash,
                message: `Amount must be between $${minimumPayout.toFixed(2)} and $${availableCash.toFixed(2)}`,
              },
            ]}
            initialValue={availableCash}
          >
            <Input
              type="number"
              prefix="$"
              step={0.01}
              min={minimumPayout}
              max={availableCash}
            />
          </FormItem>
          <Alert
            type="info"
            message={`Payout will be sent to ${paypalConnection?.email}`}
            style={{ marginTop: 12 }}
          />
        </Form>
      </Modal>
    </Card>
  );
}
