import { useState } from 'react';
import {
  Form,
  Input,
  Button,
  Space,
  Divider,
  message,
} from 'antd';
import { GoogleCircleFilled } from '@ant-design/icons';
import { useAuthStore } from '../../store/authStore';
import { useNavigate } from 'react-router-dom';
import './LoginForm.css';

export function LoginForm() {
  const [form] = Form.useForm();
  const navigate = useNavigate();
  const { loginWithEmail, loginWithOAuth, isLoading, error, clearError } =
    useAuthStore();
  const [mode, setMode] = useState<'login' | 'signup'>('login');

  const handleEmailSubmit = async (values: {
    email: string;
    password: string;
    name?: string;
  }) => {
    clearError();
    try {
      await loginWithEmail(values.email, values.password);
      message.success(`${mode === 'login' ? 'Login' : 'Sign up'} successful!`);
      navigate('/home');
    } catch (err) {
      message.error(
        err instanceof Error ? err.message : 'Authentication failed'
      );
    }
  };

  const handleOAuthLogin = async (provider: 'google' | 'apple') => {
    clearError();
    try {
      await loginWithOAuth(provider);
      message.success(`Successfully logged in with ${provider}!`);
      navigate('/home');
    } catch (err) {
      message.error(
        err instanceof Error ? err.message : `${provider} login failed`
      );
    }
  };

  return (
    <div className="login-form-container">
      <div className="login-form-card">
        <div className="login-header">
          <h1 className="login-title">
            {mode === 'login' ? 'Welcome Back' : 'Create Account'}
          </h1>
          <p className="login-subtitle">
            {mode === 'login'
              ? 'Sign in to continue to HiNotes'
              : 'Sign up to get started with HiNotes'}
          </p>
        </div>

        <Space direction="vertical" size="large" className="login-oauth-buttons" style={{ width: '100%' }}>
          <Button
            type="default"
            size="large"
            block
            icon={<GoogleCircleFilled />}
            onClick={() => handleOAuthLogin('google')}
            loading={isLoading}
            className="oauth-button google-button"
          >
            Continue with Google
          </Button>

          <Button
            type="default"
            size="large"
            block
            icon={
              <span className="apple-icon" role="img" aria-label="Apple">

              </span>
            }
            onClick={() => handleOAuthLogin('apple')}
            loading={isLoading}
            className="oauth-button apple-button"
          >
            Continue with Apple
          </Button>
        </Space>

        <Divider className="login-divider">or</Divider>

        {error && (
          <div className="login-error" role="alert">
            {error}
          </div>
        )}

        <Form
          form={form}
          layout="vertical"
          onFinish={handleEmailSubmit}
          className="login-form"
        >
          {mode === 'signup' && (
            <Form.Item
              label="Name"
              name="name"
              rules={[{ required: true, message: 'Please enter your name' }]}
            >
              <Input
                placeholder="Enter your name"
                size="large"
                disabled={isLoading}
              />
            </Form.Item>
          )}

          <Form.Item
            label="Email"
            name="email"
            rules={[
              { required: true, message: 'Please enter your email' },
              {
                type: 'email',
                message: 'Please enter a valid email address',
              },
            ]}
          >
            <Input
              placeholder="Enter your email"
              size="large"
              disabled={isLoading}
            />
          </Form.Item>

          <Form.Item
            label="Password"
            name="password"
            rules={[
              { required: true, message: 'Please enter your password' },
              {
                min: 8,
                message: 'Password must be at least 8 characters',
              },
            ]}
          >
            <Input.Password
              placeholder="Enter your password"
              size="large"
              disabled={isLoading}
            />
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              size="large"
              block
              loading={isLoading}
              className="submit-button"
            >
              {mode === 'login' ? 'Sign In' : 'Sign Up'}
            </Button>
          </Form.Item>
        </Form>

        <div className="login-footer">
          <span className="footer-text">
            {mode === 'login'
              ? "Don't have an account?"
              : 'Already have an account?'}
          </span>
          <Button
            type="link"
            onClick={() => setMode(mode === 'login' ? 'signup' : 'login')}
            disabled={isLoading}
            className="toggle-mode-button"
          >
            {mode === 'login' ? 'Sign Up' : 'Sign In'}
          </Button>
        </div>
      </div>
    </div>
  );
}
