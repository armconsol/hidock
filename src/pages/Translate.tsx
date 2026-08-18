import { useState } from 'react';
import { Card, Tabs, Space, Button, Message } from '@arco-design/web-react';
import { IconSettings, IconDelete } from '@arco-design/web-react/icon';
import { TranslationPanel } from '../components/Translation/TranslationPanel';
import { TranslationSettings } from '../components/Translation/TranslationSettings';
import { useTranslationStore } from '../store/translationStore';
import './Translate.css';

const { TabPane } = Tabs;

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
    clearError,
  } = useTranslationStore();

  const handleTranslate = async (text: string, sourceLang: string, targetLang: string) => {
    await translateText(text, sourceLang, targetLang);
  };

  const handleCopy = () => {
    Message.success('Translated text copied to clipboard');
  };

  const handleClearCache = async () => {
    await clearCache();
    Message.success('Translation cache cleared');
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
            icon={<IconDelete />}
            onClick={handleClearCache}
            type="outline"
          >
            Clear Cache
          </Button>
        </Space>
      </div>

      <Tabs activeTab={activeTab} onChange={setActiveTab}>
        <TabPane key="translate" title="Translate">
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
        </TabPane>

        <TabPane key="settings" title={<><IconSettings /> Settings</>}>
          <TranslationSettings
            settings={settings}
            onSave={handleSaveSettings}
          />
        </TabPane>
      </Tabs>
    </div>
  );
}
