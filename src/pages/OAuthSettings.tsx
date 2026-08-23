import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Card,
  Form,
  Input,
  Button,
  message,
  Space,
  Typography,
  Collapse,
  List,
} from 'antd';
import { InfoCircleOutlined, CopyOutlined, CheckOutlined } from '@ant-design/icons';

const { Title, Paragraph, Text } = Typography;

interface AppConfig {
  google_client_id?: string;
  google_client_secret?: string;
  apple_client_id?: string;
  apple_team_id?: string;
  apple_key_id?: string;
  api_base_url?: string;
}

interface OAuthInstructions {
  provider: string;
  title: string;
  steps: string[];
  required_fields: string[];
  documentation_url: string;
}

export default function OAuthSettings() {
  const [config, setConfig] = useState<AppConfig>({});
  const [loading, setLoading] = useState(false);
  const [configPath, setConfigPath] = useState<string>('');
  const [googleInstructions, setGoogleInstructions] = useState<OAuthInstructions | null>(null);
  const [appleInstructions, setAppleInstructions] = useState<OAuthInstructions | null>(null);
  const [copiedField, setCopiedField] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
    loadInstructions();
    loadConfigPath();
  }, []);

  const loadConfig = async () => {
    try {
      const loadedConfig = await invoke<AppConfig>('load_config');
      setConfig(loadedConfig);
    } catch (error) {
      message.error(`Failed to load config: ${error}`);
    }
  };

  const loadInstructions = async () => {
    try {
      const google = await invoke<OAuthInstructions>('get_google_oauth_instructions');
      const apple = await invoke<OAuthInstructions>('get_apple_oauth_instructions');
      setGoogleInstructions(google);
      setAppleInstructions(apple);
    } catch (error) {
      message.error(`Failed to load instructions: ${error}`);
    }
  };

  const loadConfigPath = async () => {
    try {
      const path = await invoke<string>('get_config_file_path');
      setConfigPath(path);
    } catch (error) {
      console.error('Failed to load config path:', error);
    }
  };

  const handleSave = async () => {
    setLoading(true);
    try {
      await invoke('save_config', { config });
      message.success('Configuration saved successfully! Please restart the application for changes to take effect.');
    } catch (error) {
      message.error(`Failed to save config: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = (text: string, field: string) => {
    navigator.clipboard.writeText(text);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  const renderInstructions = (instructions: OAuthInstructions | null, provider: string) => {
    if (!instructions) return null;

    return (
      <Collapse.Panel
        header={
          <Space>
            <InfoCircleOutlined />
            {instructions.title}
          </Space>
        }
        key={provider}
      >
        <Card bordered={false} style={{ background: '#fafafa' }}>
          <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <div>
              <Title level={5}>Required Credentials</Title>
              <List
                size="small"
                dataSource={instructions.required_fields}
                renderItem={(item) => <List.Item>{item}</List.Item>}
              />
            </div>

            <div>
              <Title level={5}>Setup Steps</Title>
              <List
                size="small"
                dataSource={instructions.steps}
                renderItem={(item) => (
                  <List.Item style={{ paddingLeft: item.startsWith('   -') ? '2em' : '0' }}>
                    <Text style={{ fontSize: '13px' }}>{item}</Text>
                  </List.Item>
                )}
              />
            </div>

            <Button
              type="default"
              onClick={() => window.open(instructions.documentation_url, '_blank')}
            >
              View Official Documentation
            </Button>
          </Space>
        </Card>
      </Collapse.Panel>
    );
  };

  return (
    <div style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <div>
          <Title level={3}>OAuth Configuration</Title>
          <Paragraph>
            Configure OAuth credentials to enable Google Sign-In and Apple Sign-In features.
            The application must be restarted after saving changes.
          </Paragraph>
          {configPath && (
            <Paragraph type="secondary">
              Configuration file location:{' '}
              <Text code copyable={{ onCopy: () => copyToClipboard(configPath, 'path') }}>
                {configPath}
              </Text>
            </Paragraph>
          )}
        </div>

        <Card title="Setup Instructions" bordered>
          <Collapse accordion>
            {renderInstructions(googleInstructions, 'google')}
            {renderInstructions(appleInstructions, 'apple')}
          </Collapse>
        </Card>

        <Card title="Google OAuth Configuration" bordered>
          <Form layout="vertical">
            <Form.Item
              label="Google Client ID"
              required
              extra="Required for Google Sign-In. Format: xxxxx.apps.googleusercontent.com"
            >
              <Input
                value={config.google_client_id || ''}
                onChange={(e) => setConfig({ ...config, google_client_id: e.target.value })}
                placeholder="xxxxx.apps.googleusercontent.com"
                suffix={
                  config.google_client_id && (
                    <Button
                      type="text"
                      size="small"
                      icon={copiedField === 'google_id' ? <CheckOutlined /> : <CopyOutlined />}
                      onClick={() => copyToClipboard(config.google_client_id!, 'google_id')}
                    />
                  )
                }
              />
            </Form.Item>

            <Form.Item
              label="Google Client Secret"
              extra="Optional, but recommended for better security"
            >
              <Input.Password
                value={config.google_client_secret || ''}
                onChange={(e) => setConfig({ ...config, google_client_secret: e.target.value })}
                placeholder="Enter Google Client Secret"
              />
            </Form.Item>
          </Form>
        </Card>

        <Card title="Apple Sign In Configuration (Optional)" bordered>
          <Form layout="vertical">
            <Form.Item
              label="Apple Client ID (Service ID)"
              extra="Service ID from Apple Developer Portal"
            >
              <Input
                value={config.apple_client_id || ''}
                onChange={(e) => setConfig({ ...config, apple_client_id: e.target.value })}
                placeholder="com.yourcompany.hinotes.signin"
              />
            </Form.Item>

            <Form.Item label="Apple Team ID" extra="10-character Team ID">
              <Input
                value={config.apple_team_id || ''}
                onChange={(e) => setConfig({ ...config, apple_team_id: e.target.value })}
                placeholder="ABC1234567"
                maxLength={10}
              />
            </Form.Item>

            <Form.Item label="Apple Key ID" extra="10-character Key ID from your .p8 file">
              <Input
                value={config.apple_key_id || ''}
                onChange={(e) => setConfig({ ...config, apple_key_id: e.target.value })}
                placeholder="XYZ9876543"
                maxLength={10}
              />
            </Form.Item>
          </Form>
        </Card>

        <Card title="Advanced Settings" bordered>
          <Form layout="vertical">
            <Form.Item
              label="API Base URL"
              extra="Leave empty to use default: https://hinotes.hidock.com/v1"
            >
              <Input
                value={config.api_base_url || ''}
                onChange={(e) => setConfig({ ...config, api_base_url: e.target.value })}
                placeholder="https://hinotes.hidock.com/v1"
              />
            </Form.Item>
          </Form>
        </Card>

        <Space>
          <Button type="primary" loading={loading} onClick={handleSave}>
            Save Configuration
          </Button>
          <Button onClick={loadConfig}>Reset to Saved</Button>
        </Space>
      </Space>
    </div>
  );
}
