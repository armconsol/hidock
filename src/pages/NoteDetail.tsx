import { useState } from 'react';
import { Card, Button, Space, Divider } from 'antd';
import { TranslationOutlined } from '@ant-design/icons';
import { InlineTranslation } from '../components/Translation/InlineTranslation';
import { useTranslationStore } from '../store/translationStore';
import './NoteDetail.css';

export function NoteDetailPage() {
  const [noteContent] = useState('Hello world. This is a sample note content that can be translated.');
  const { translateText, settings } = useTranslationStore();
  const [translations, setTranslations] = useState<Map<string, string>>(new Map());
  const [loadingTranslations, setLoadingTranslations] = useState<Set<string>>(new Set());

  const handleTranslate = async (text: string) => {
    setLoadingTranslations((prev) => new Set(prev).add(text));

    try {
      await translateText(text, settings.defaultSourceLang, settings.defaultTargetLang);
      // In a real implementation, you would get the translated text from the store
      // and update the translations map
      setTranslations((prev) => {
        const newMap = new Map(prev);
        newMap.set(text, 'Translated: ' + text); // Mock translation
        return newMap;
      });
    } finally {
      setLoadingTranslations((prev) => {
        const newSet = new Set(prev);
        newSet.delete(text);
        return newSet;
      });
    }
  };

  return (
    <div className="note-detail-page">
      <Card
        title="Note Detail"
        extra={
          <Button type="primary" icon={<TranslationOutlined />}>
            Translate All
          </Button>
        }
      >
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          <div className="note-header">
            <h2>Sample Note Title</h2>
            <p className="note-meta">Created: 2026-08-18 | Updated: 2026-08-18</p>
          </div>

          <Divider />

          <div className="note-content">
            <InlineTranslation
              text={noteContent}
              translatedText={translations.get(noteContent)}
              targetLang={settings.defaultTargetLang}
              isLoading={loadingTranslations.has(noteContent)}
              onTranslate={handleTranslate}
            />
          </div>

          <div className="note-paragraphs">
            {noteContent.split('. ').map((sentence, index) => (
              <div key={index} className="note-paragraph">
                <InlineTranslation
                  text={sentence}
                  translatedText={translations.get(sentence)}
                  targetLang={settings.defaultTargetLang}
                  isLoading={loadingTranslations.has(sentence)}
                  onTranslate={handleTranslate}
                />
              </div>
            ))}
          </div>
        </Space>
      </Card>
    </div>
  );
}
