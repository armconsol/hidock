import { useState } from 'react';
import { Tabs, Space, Button, message } from 'antd';
import { SettingOutlined, DeleteOutlined } from '@ant-design/icons';
import { TranslationPanel } from '../components/Translation/TranslationPanel';
import { TranslationSettings } from '../components/Translation/TranslationSettings';
import { useTranslationStore } from '../store/translationStore';
import './Translate.css';

export function TranslatePage() {
  const [activeTab, setActiveTab] = useState('translate');
  const {
    isLoading,
    error,
    translatedText,
    confidence,
    settings,
    translateText,
    clearCache,
    saveSettings,
  } = useTranslationStore();

  const handleTranslate = async (text: string, sourceLang: string, targetLang: string) => {
    await translateText(text, sourceLang, targetLang);
  };

  const handleCopy = () => {
    message.success('Translated text copied to clipboard');
  };

  const handleClearCache = async () => {
    await clearCache();
    message.success('Translation cache cleared');
  };

  const handleSaveSettings = (newSettings: typeof settings) => {
    saveSettings(newSettings);
  };

  return (
    <div className="translate-page">
      <div className="translate-header">
        <h1>Translation</h1>
        <Space>
          <Button
            icon={<DeleteOutlined />}
            onClick={handleClearCache}
          >
            Clear Cache
          </Button>
        </Space>
      </div>

      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        items={[
          {
            key: 'translate',
            label: 'Translate',
            children: (
              <TranslationPanel
                sourceLang={settings.defaultSourceLang}
                targetLang={settings.defaultTargetLang}
                translatedText={translatedText ?? undefined}
                confidence={confidence ?? undefined}
                isLoading={isLoading}
                error={error ?? undefined}
                onTranslate={handleTranslate}
                onCopy={handleCopy}
              />
            ),
          },
          {
            key: 'settings',
            label: (
              <>
                <SettingOutlined /> Settings
              </>
            ),
            children: (
              <TranslationSettings
                settings={settings}
                onSave={handleSaveSettings}
              />
            ),
          },
        ]}
      />
    </div>
  );
}
