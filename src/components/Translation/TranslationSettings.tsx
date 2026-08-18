import { useState, useEffect } from 'react';
import { Card, Button, Switch, Space, Message } from '@arco-design/web-react';
import { LanguageSelector } from './LanguageSelector';
import { TranslationSettings as ITranslationSettings, DEFAULT_SETTINGS } from './types';
import './TranslationSettings.css';

interface TranslationSettingsProps {
  settings?: ITranslationSettings;
  onSave?: (settings: ITranslationSettings) => void;
}

export function TranslationSettings({
  settings = DEFAULT_SETTINGS,
  onSave,
}: TranslationSettingsProps) {
  const [currentSettings, setCurrentSettings] = useState<ITranslationSettings>(settings);
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    setCurrentSettings(settings);
    setHasChanges(false);
  }, [settings]);

  const handleSourceLangChange = (lang: string) => {
    setCurrentSettings({ ...currentSettings, defaultSourceLang: lang });
    setHasChanges(true);
  };

  const handleTargetLangChange = (lang: string) => {
    setCurrentSettings({ ...currentSettings, defaultTargetLang: lang });
    setHasChanges(true);
  };

  const handleAutoTranslateChange = (checked: boolean) => {
    setCurrentSettings({ ...currentSettings, autoTranslate: checked });
    setHasChanges(true);
  };

  const handleSave = () => {
    if (onSave) {
      onSave(currentSettings);
    }
    setHasChanges(false);
    Message.success('Settings saved successfully');
  };

  const handleReset = () => {
    setCurrentSettings(DEFAULT_SETTINGS);
    setHasChanges(true);
  };

  return (
    <Card title="Translation Settings" className="translation-settings">
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <div className="setting-item">
          <label>Default Source Language</label>
          <LanguageSelector
            value={currentSettings.defaultSourceLang}
            onChange={handleSourceLangChange}
            showSearch
          />
        </div>

        <div className="setting-item">
          <label>Default Target Language</label>
          <LanguageSelector
            value={currentSettings.defaultTargetLang}
            onChange={handleTargetLangChange}
            showSearch
          />
        </div>

        <div className="setting-item">
          <label>Auto-translate Notes</label>
          <Switch
            checked={currentSettings.autoTranslate}
            onChange={handleAutoTranslateChange}
          />
          <div className="setting-description">
            Automatically translate notes when opened
          </div>
        </div>

        <Space>
          <Button type="primary" onClick={handleSave} disabled={!hasChanges}>
            Save Settings
          </Button>
          <Button onClick={handleReset}>Reset to Defaults</Button>
        </Space>
      </Space>
    </Card>
  );
}
