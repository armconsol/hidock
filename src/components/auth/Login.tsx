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
} from '@arco-design/web-react';
import {
  IconGoogleCircleFill,
  IconUser,
  IconEmail,
  IconLock,
} from '@arco-design/web-react/icon';
import { useAuthStore } from '../../store/authStore';
import './Login.css';

const { Title, Text } = Typography;
const FormItem = Form.Item;

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
          <Title heading={3}>Welcome to HiNotes</Title>
          <Text type="secondary">Sign in to continue</Text>
        </div>

        {error && (
          <Alert
            type="error"
            closable
            onClose={clearError}
            style={{ marginBottom: 20 }}
            content={error}
          />
        )}

        {!isEmailMode ? (
          <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Button
              type="primary"
              size="large"
              long
              icon={<IconGoogleCircleFill />}
              loading={isLoading}
              onClick={() => handleOAuthLogin('google')}
              className="oauth-button oauth-button-google"
            >
              Continue with Google
            </Button>

            <Button
              type="primary"
              size="large"
              long
              icon={<IconUser />}
              loading={isLoading}
              onClick={() => handleOAuthLogin('apple')}
              className="oauth-button oauth-button-apple"
            >
              Continue with Apple
            </Button>

            <Divider>or</Divider>

            <Button
              type="outline"
              size="large"
              long
              icon={<IconEmail />}
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
              onSubmit={handleEmailLogin}
              autoComplete="off"
            >
              <FormItem
                label="Email"
                field="email"
                rules={[
                  { required: true, message: 'Email is required' },
                  { type: 'email', message: 'Invalid email format' },
                ]}
              >
                <Input
                  prefix={<IconEmail />}
                  placeholder="Enter your email"
                  size="large"
                  disabled={isLoading}
                />
              </FormItem>

              <FormItem
                label="Password"
                field="password"
                rules={[
                  { required: true, message: 'Password is required' },
                  { minLength: 6, message: 'Password must be at least 6 characters' },
                ]}
              >
                <Input.Password
                  prefix={<IconLock />}
                  placeholder="Enter your password"
                  size="large"
                  disabled={isLoading}
                />
              </FormItem>

              <FormItem>
                <Space direction="vertical" size="medium" style={{ width: '100%' }}>
                  <Button
                    type="primary"
                    size="large"
                    long
                    htmlType="submit"
                    loading={isLoading}
                  >
                    Sign In
                  </Button>
                  <Button
                    type="text"
                    size="large"
                    long
                    onClick={() => setIsEmailMode(false)}
                    disabled={isLoading}
                  >
                    Back to other options
                  </Button>
                </Space>
              </FormItem>
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
