import { useState, useEffect } from 'react';
import { Card, Button, Input, Spin, Alert, Space } from 'antd';
import {
  SwapOutlined,
  CopyOutlined,
  TranslationOutlined,
} from '@ant-design/icons';
import { LanguageSelector } from './LanguageSelector';
import './TranslationPanel.css';

const { TextArea } = Input;

interface TranslationPanelProps {
  initialText?: string;
  translatedText?: string;
  sourceLang?: string;
  targetLang?: string;
  confidence?: number;
  isLoading?: boolean;
  error?: string;
  layout?: 'side-by-side' | 'stacked';
  maxLength?: number;
  onTranslate?: (text: string, sourceLang: string, targetLang: string) => void;
  onCopy?: () => void;
  onSwapLanguages?: () => void;
}

export function TranslationPanel({
  initialText = '',
  translatedText = '',
  sourceLang = 'en',
  targetLang = 'es',
  confidence,
  isLoading = false,
  error,
  layout = 'side-by-side',
  maxLength = 5000,
  onTranslate,
  onCopy,
  onSwapLanguages,
}: TranslationPanelProps) {
  const [sourceText, setSourceText] = useState(initialText);
  const [currentSourceLang, setCurrentSourceLang] = useState(sourceLang);
  const [currentTargetLang, setCurrentTargetLang] = useState(targetLang);

  useEffect(() => {
    setSourceText(initialText);
  }, [initialText]);

  const handleTextChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const truncated = e.target.value.slice(0, maxLength);
    setSourceText(truncated);
  };

  const handleSourceLangChange = (lang: string) => {
    setCurrentSourceLang(lang);
    if (onTranslate && sourceText) {
      onTranslate(sourceText, lang, currentTargetLang);
    }
  };

  const handleTargetLangChange = (lang: string) => {
    setCurrentTargetLang(lang);
    if (onTranslate && sourceText) {
      onTranslate(sourceText, currentSourceLang, lang);
    }
  };

  const handleSwapLanguages = () => {
    const tempLang = currentSourceLang;
    setCurrentSourceLang(currentTargetLang);
    setCurrentTargetLang(tempLang);
    if (onSwapLanguages) {
      onSwapLanguages();
    }
    if (onTranslate && sourceText) {
      onTranslate(sourceText, currentTargetLang, tempLang);
    }
  };

  const handleCopy = async () => {
    if (translatedText) {
      await navigator.clipboard.writeText(translatedText);
      if (onCopy) {
        onCopy();
      }
    }
  };

  const handleTranslate = () => {
    if (onTranslate && sourceText) {
      onTranslate(sourceText, currentSourceLang, currentTargetLang);
    }
  };

  return (
    <div className={`translation-panel ${layout}`} data-testid="translation-panel">
      {error && (
        <Alert type="error" message={error} closable style={{ marginBottom: 16 }} />
      )}

      <div className="language-selectors">
        <div className="language-selector-wrapper">
          <label>Source Language</label>
          <LanguageSelector
            value={currentSourceLang}
            onChange={handleSourceLangChange}
            showSearch
          />
        </div>

        <Button
          type="text"
          icon={<SwapOutlined />}
          onClick={handleSwapLanguages}
          aria-label="Swap languages"
          className="swap-button"
        />

        <div className="language-selector-wrapper">
          <label>Target Language</label>
          <LanguageSelector
            value={currentTargetLang}
            onChange={handleTargetLangChange}
            showSearch
          />
        </div>
      </div>

      <div className="translation-content">
        <Card
          title="Original Text"
          className="source-card"
          extra={
            <span className="char-count">
              {sourceText.length} / {maxLength} characters
            </span>
          }
        >
          <TextArea
            value={sourceText}
            onChange={handleTextChange}
            placeholder="Enter text to translate..."
            autoSize={{ minRows: 6, maxRows: 12 }}
            maxLength={maxLength}
          />
          <Button
            type="primary"
            icon={<TranslationOutlined />}
            onClick={handleTranslate}
            disabled={!sourceText || isLoading}
            style={{ marginTop: 12 }}
          >
            Translate
          </Button>
        </Card>

        <Card
          title="Translated Text"
          className="target-card"
          extra={
            <Space>
              {confidence !== undefined && (
                <span
                  className="confidence-badge"
                  style={{
                    padding: '2px 8px',
                    borderRadius: '12px',
                    fontSize: '12px',
                    fontWeight: 'bold',
                    color: 'white',
                    backgroundColor:
                      confidence > 0.9
                        ? '#00b42a'
                        : confidence > 0.7
                        ? '#ff7d00'
                        : '#f53f3f',
                  }}
                >
                  {Math.round(confidence * 100)}%
                </span>
              )}
              <Button
                type="text"
                icon={<CopyOutlined />}
                onClick={handleCopy}
                disabled={!translatedText}
                aria-label="Copy translated text"
              >
                Copy
              </Button>
            </Space>
          }
        >
          {isLoading ? (
            <div className="loading-container">
              <Spin tip="Translating..." />
            </div>
          ) : (
            <div className="translated-text">
              {translatedText || 'Translation will appear here...'}
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
