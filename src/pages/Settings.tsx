import { useState, useEffect } from 'react';
import {
  Card,
  Switch,
  Select,
  Radio,
  Alert,
  Button,
  Space,
  Typography,
  Divider,
  message,
  Spin,
} from 'antd';
import {
  SyncOutlined,
  CheckOutlined,
  CloseOutlined,
  ClockCircleOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { useSyncStore } from '../store/syncStore';
import { useSettingsStore } from '../store/settingsStore';
import { SyncIndicator } from '../components/Sync/SyncIndicator';
import './Settings.css';

const { Title, Text, Paragraph } = Typography;

interface AIEngine {
  id: string;
  name: string;
  description: string;
  capabilities: string[];
  is_default: boolean;
  requires_subscription: boolean;
}

interface SyncSettingsResponse {
  synced_count: number;
  conflicts_resolved: number;
  errors: string[];
}

interface ConflictData {
  key: string;
  localValue: string;
  cloudValue: string;
}

export function SettingsPage() {
  const { theme, setTheme } = useSettingsStore();
  const { status: syncStatus, lastSyncTime, error: syncError } = useSyncStore();

  // State
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [aiEngines, setAiEngines] = useState<AIEngine[]>([]);
  const [selectedEngine, setSelectedEngine] = useState<string>('');
  const [cloudSyncEnabled, setCloudSyncEnabled] = useState(true);
  const [conflicts, setConflicts] = useState<ConflictData[]>([]);
  const [lastSyncResult, setLastSyncResult] = useState<SyncSettingsResponse | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    setLoading(true);
    setSettingsError(null);

    try {
      // Load AI engines
      try {
        const engines = await invoke<AIEngine[]>('get_ai_engines');
        setAiEngines(engines);

        // Load current engine setting
        const currentEngine = await invoke<string | null>('get_user_setting', {
          key: 'transcription_engine',
        });

        if (currentEngine) {
          setSelectedEngine(currentEngine);
        } else if (engines.length > 0) {
          // Set default engine
          const defaultEngine = engines.find((e) => e.is_default);
          if (defaultEngine) {
            setSelectedEngine(defaultEngine.id);
          }
        }
      } catch (error) {
        console.warn('Failed to load AI engines:', error);
        // Non-fatal - user might not be authenticated
      }

      // Load cloud sync setting
      const cloudSyncSetting = await invoke<string | null>('get_user_setting', {
        key: 'cloud_sync_enabled',
      });
      if (cloudSyncSetting !== null) {
        setCloudSyncEnabled(cloudSyncSetting === 'true');
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      setSettingsError(`Failed to load settings: ${errorMessage}`);
      message.error('Failed to load settings');
      console.error('Error loading settings:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSyncNow = async () => {
    if (!cloudSyncEnabled) {
      message.warning('Cloud sync is disabled');
      return;
    }

    setSyncing(true);
    setSettingsError(null);

    try {
      const result = await invoke<SyncSettingsResponse>('sync_settings_with_cloud');
      setLastSyncResult(result);

      if (result.errors.length > 0) {
        message.warning(
          `Sync completed with ${result.errors.length} error(s). Check details below.`
        );
      } else {
        message.success(
          `Synced ${result.synced_count} setting(s)${
            result.conflicts_resolved > 0
              ? `, resolved ${result.conflicts_resolved} conflict(s)`
              : ''
          }`
        );
      }

      // Reload settings to reflect synced changes
      await loadSettings();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      setSettingsError(`Sync failed: ${errorMessage}`);
      Message.error('Failed to sync settings');
      console.error('Error syncing settings:', error);
    } finally {
      setSyncing(false);
    }
  };

  const handleEngineChange = async (engineId: string) => {
    setSelectedEngine(engineId);

    try {
      await invoke('set_user_setting', {
        key: 'transcription_engine',
        value: engineId,
        syncToCloud: cloudSyncEnabled,
      });

      message.success('AI engine updated');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      Message.error(`Failed to update AI engine: ${errorMessage}`);
      console.error('Error updating AI engine:', error);
    }
  };

  const handleCloudSyncToggle = async (enabled: boolean) => {
    setCloudSyncEnabled(enabled);

    try {
      await invoke('set_user_setting', {
        key: 'cloud_sync_enabled',
        value: String(enabled),
        syncToCloud: false, // Don't sync this setting itself
      });

      if (enabled) {
        message.success('Cloud sync enabled');
        // Auto-sync on enable
        await handleSyncNow();
      } else {
        message.info('Cloud sync disabled - settings will stay local only');
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      Message.error(`Failed to update cloud sync setting: ${errorMessage}`);
      console.error('Error updating cloud sync:', error);
    }
  };

  const handleThemeChange = async (newTheme: 'light' | 'dark') => {
    setTheme(newTheme);

    try {
      await invoke('set_user_setting', {
        key: 'theme',
        value: newTheme,
        syncToCloud: cloudSyncEnabled,
      });
    } catch (error) {
      console.error('Error saving theme setting:', error);
    }
  };

  const handleConflictResolve = async (conflict: ConflictData, useCloud: boolean) => {
    try {
      const valueToUse = useCloud ? conflict.cloudValue : conflict.localValue;

      await invoke('set_user_setting', {
        key: conflict.key,
        value: valueToUse,
        syncToCloud: useCloud,
      });

      // Remove resolved conflict
      setConflicts((prev) => prev.filter((c) => c.key !== conflict.key));

      message.success(
        `Conflict resolved: using ${useCloud ? 'cloud' : 'local'} value for ${conflict.key}`
      );
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      Message.error(`Failed to resolve conflict: ${errorMessage}`);
      console.error('Error resolving conflict:', error);
    }
  };

  const formatLastSyncTime = (date: Date | null): string => {
    if (!date) return 'Never synced';

    const syncDate = new Date(date);
    const now = new Date();
    const diffMs = now.getTime() - syncDate.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
  };

  if (loading) {
    return (
      <div className="settings-page loading">
        <Spin size="large" />
      </div>
    );
  }

  return (
    <div className="settings-page">
      <div className="settings-header">
        <Title level={1}>Settings</Title>
        <SyncIndicator />
      </div>

      {/* Sync Status Section */}
      <Card className="settings-section sync-status-section">
        <div className="section-header">
          <Title level={3}>
            <SyncOutlined style={{ marginRight: 8 }} />
            Cloud Sync Status
          </Title>
          <Button
            type="primary"
            icon={syncing ? <ReloadOutlined className="spinning" /> : <SyncOutlined />}
            loading={syncing}
            disabled={!cloudSyncEnabled || syncing}
            onClick={handleSyncNow}
          >
            Sync Now
          </Button>
        </div>

        <Space direction="vertical" size="medium" style={{ width: '100%' }}>
          {/* Last sync time */}
          <div className="sync-info-row">
            <Text type="secondary">
              <ClockCircleOutlined style={{ marginRight: 6 }} />
              Last synced:
            </Text>
            <Text>{formatLastSyncTime(lastSyncTime)}</Text>
          </div>

          {/* Sync status indicator */}
          <div className="sync-info-row">
            <Text type="secondary">Status:</Text>
            <Space>
              {syncStatus === 'synced' && (
                <>
                  <CheckOutlined style={{ color: 'rgb(var(--success-6))' }} />
                  <Text>All changes synced</Text>
                </>
              )}
              {syncStatus === 'syncing' && (
                <>
                  <SyncOutlined className="spinning" style={{ color: 'rgb(var(--primary-6))' }} />
                  <Text>Syncing...</Text>
                </>
              )}
              {syncStatus === 'offline' && (
                <>
                  <Text type="secondary">Offline</Text>
                </>
              )}
              {syncStatus === 'failed' && (
                <>
                  <CloseOutlined style={{ color: 'rgb(var(--danger-6))' }} />
                  <Text type="danger">Sync failed</Text>
                </>
              )}
            </Space>
          </div>

          {/* Last sync result */}
          {lastSyncResult && (
            <Alert
              type={lastSyncResult.errors.length > 0 ? 'warning' : 'success'}
              message={
                <Space direction="vertical" size="small">
                  <Text>
                    Synced {lastSyncResult.synced_count} setting(s)
                    {lastSyncResult.conflicts_resolved > 0 &&
                      `, resolved ${lastSyncResult.conflicts_resolved} conflict(s)`}
                  </Text>
                  {lastSyncResult.errors.length > 0 && (
                    <div>
                      <Text type="danger">Errors:</Text>
                      <ul style={{ margin: '4px 0 0 0', paddingLeft: '20px' }}>
                        {lastSyncResult.errors.map((err, idx) => (
                          <li key={idx}>
                            <Text type="danger">{err}</Text>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </Space>
              }
              closable
              onClose={() => setLastSyncResult(null)}
            />
          )}

          {/* Sync error */}
          {(syncError || settingsError) && (
            <Alert
              type="error"
              message={
                <Space direction="vertical" size="small">
                  <Text type="danger">{syncError || settingsError}</Text>
                  <Button
                    type="text"
                    size="small"
                    onClick={handleSyncNow}
                    disabled={!cloudSyncEnabled}
                  >
                    Retry
                  </Button>
                </Space>
              }
              closable
              onClose={() => setSettingsError(null)}
            />
          )}
        </Space>
      </Card>

      {/* Conflicts Section */}
      {conflicts.length > 0 && (
        <Card className="settings-section conflicts-section">
          <Title level={3}>Sync Conflicts</Title>
          <Paragraph type="secondary">
            The following settings have different values locally and in the cloud. Choose which
            value to keep.
          </Paragraph>

          <Space direction="vertical" size="medium" style={{ width: '100%' }}>
            {conflicts.map((conflict) => (
              <Card key={conflict.key} className="conflict-item">
                <Title level={4}>{conflict.key}</Title>
                <div className="conflict-comparison">
                  <div className="conflict-value">
                    <Text type="secondary">Local:</Text>
                    <Text code>{conflict.localValue}</Text>
                    <Button
                      type="primary"
                      size="small"
                      onClick={() => handleConflictResolve(conflict, false)}
                    >
                      Keep Local
                    </Button>
                  </div>
                  <Divider type="vertical" style={{ height: '60px' }} />
                  <div className="conflict-value">
                    <Text type="secondary">Cloud:</Text>
                    <Text code>{conflict.cloudValue}</Text>
                    <Button
                      type="primary"
                      size="small"
                      onClick={() => handleConflictResolve(conflict, true)}
                    >
                      Use Cloud
                    </Button>
                  </div>
                </div>
              </Card>
            ))}
          </Space>
        </Card>
      )}

      {/* General Settings Section */}
      <Card className="settings-section general-settings-section">
        <Title heading={3}>General Settings</Title>

        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {/* Cloud Sync Toggle */}
          <div className="setting-row">
            <div className="setting-label">
              <Text>Cloud Sync</Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                Automatically sync settings across devices
              </Text>
            </div>
            <Switch
              checked={cloudSyncEnabled}
              onChange={handleCloudSyncToggle}
              checkedChildren="On"
              unCheckedChildren="Off"
            />
          </div>

          <Divider />

          {/* Theme Setting */}
          <div className="setting-row">
            <div className="setting-label">
              <Text>Theme</Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                Choose your preferred color scheme
              </Text>
            </div>
            <Radio.Group
              optionType="button"
              value={theme}
              onChange={(e) => handleThemeChange(e.target.value as 'light' | 'dark')}
            >
              <Radio value="light">Light</Radio>
              <Radio value="dark">Dark</Radio>
            </Radio.Group>
          </div>

          <Divider />

          {/* AI Engine Setting */}
          <div className="setting-row">
            <div className="setting-label">
              <Text>AI Transcription Engine</Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                Select the AI engine for audio transcription
              </Text>
            </div>
            {aiEngines && aiEngines.length > 0 ? (
              <Select
                placeholder="Select AI engine"
                value={selectedEngine}
                onChange={handleEngineChange}
                style={{ width: 280 }}
                disabled={!cloudSyncEnabled && aiEngines.length === 0}
              >
                {aiEngines.map((engine) => (
                  <Select.Option key={engine.id} value={engine.id}>
                    <Space direction="vertical" size={0}>
                      <Text>
                        {engine.name}
                        {engine.is_default && (
                          <Text type="secondary" style={{ fontSize: 11, marginLeft: 6 }}>
                            (Default)
                          </Text>
                        )}
                      </Text>
                      <Text type="secondary" style={{ fontSize: 11 }}>
                        {engine.description}
                      </Text>
                      {engine.requires_subscription && (
                        <Text type="warning" style={{ fontSize: 11 }}>
                          Requires subscription
                        </Text>
                      )}
                    </Space>
                  </Select.Option>
                ))}
              </Select>
            ) : (
              <Text type="secondary">No AI engines available. Please sign in.</Text>
            )}
          </div>

          {/* Engine capabilities */}
          {selectedEngine && aiEngines && aiEngines.length > 0 && (
            <div className="engine-capabilities">
              <Text type="secondary">Capabilities:</Text>
              <Space wrap style={{ marginTop: 8 }}>
                {aiEngines
                  .find((e) => e.id === selectedEngine)
                  ?.capabilities?.map((cap) => (
                    <div key={cap} className="capability-tag">
                      <CheckOutlined style={{ fontSize: 12, marginRight: 4 }} />
                      <Text style={{ fontSize: 12 }}>{cap}</Text>
                    </div>
                  ))}
              </Space>
            </div>
          )}
        </Space>
      </Card>
    </div>
  );
}
