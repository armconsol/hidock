import { useState } from 'react';
import {
  Card,
  Form,
  Input,
  Button,
  Space,
  Divider,
  Alert,
  Typography,
} from 'antd';
import {
  GoogleCircleFilled,
  UserOutlined,
  MailOutlined,
  LockOutlined,
} from '@ant-design/icons';
import { useAuthStore } from '../../store/authStore';
import './Login.css';

const { Title, Text } = Typography;

export const Login = () => {
  const [form] = Form.useForm();
  const { loginWithEmail, loginWithOAuth, isLoading, error, clearError } = useAuthStore();
  const [isEmailMode, setIsEmailMode] = useState(false);

  const handleOAuthLogin = async (provider: 'google' | 'apple') => {
    clearError();
    await loginWithOAuth(provider);
  };

  const handleEmailLogin = async (values: { email: string; password: string }) => {
    clearError();
    await loginWithEmail(values.email, values.password);
  };

  return (
    <div className="login-container">
      <Card className="login-card" bordered={false}>
        <div className="login-header">
          <Title level={3}>Welcome to HiNotes</Title>
          <Text type="secondary">Sign in to continue</Text>
        </div>

        {error && (
          <Alert
            type="error"
            closable
            onClose={clearError}
            style={{ marginBottom: 20 }}
            message={error}
          />
        )}

        {!isEmailMode ? (
          <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Button
              type="primary"
              size="large"
              block
              icon={<GoogleCircleFilled />}
              loading={isLoading}
              onClick={() => handleOAuthLogin('google')}
              className="oauth-button oauth-button-google"
            >
              Continue with Google
            </Button>

            <Button
              type="primary"
              size="large"
              block
              icon={<UserOutlined />}
              loading={isLoading}
              onClick={() => handleOAuthLogin('apple')}
              className="oauth-button oauth-button-apple"
            >
              Continue with Apple
            </Button>

            <Divider>or</Divider>

            <Button
              type="default"
              size="large"
              block
              icon={<MailOutlined />}
              onClick={() => setIsEmailMode(true)}
            >
              Continue with Email
            </Button>
          </Space>
        ) : (
          <>
            <Form
              form={form}
              layout="vertical"
              onFinish={handleEmailLogin}
              autoComplete="off"
            >
              <Form.Item
                label="Email"
                name="email"
                rules={[
                  { required: true, message: 'Email is required' },
                  { type: 'email', message: 'Invalid email format' },
                ]}
              >
                <Input
                  prefix={<MailOutlined />}
                  placeholder="Enter your email"
                  size="large"
                  disabled={isLoading}
                />
              </Form.Item>

              <Form.Item
                label="Password"
                name="password"
                rules={[
                  { required: true, message: 'Password is required' },
                  { min: 6, message: 'Password must be at least 6 characters' },
                ]}
              >
                <Input.Password
                  prefix={<LockOutlined />}
                  placeholder="Enter your password"
                  size="large"
                  disabled={isLoading}
                />
              </Form.Item>

              <Form.Item>
                <Space direction="vertical" size="middle" style={{ width: '100%' }}>
                  <Button
                    type="primary"
                    size="large"
                    block
                    htmlType="submit"
                    loading={isLoading}
                  >
                    Sign In
                  </Button>
                  <Button
                    type="text"
                    size="large"
                    block
                    onClick={() => setIsEmailMode(false)}
                    disabled={isLoading}
                  >
                    Back to other options
                  </Button>
                </Space>
              </Form.Item>
            </Form>
          </>
        )}

        <div className="login-footer">
          <Text type="secondary" style={{ fontSize: 12 }}>
            By continuing, you agree to our Terms of Service and Privacy Policy
          </Text>
        </div>
      </Card>
    </div>
  );
};
