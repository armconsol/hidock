import { useState } from 'react';
import { Button, Spin, Tooltip } from '@arco-design/web-react';
import { IconTranslate, IconSwap } from '@arco-design/web-react/icon';
import './InlineTranslation.css';

interface InlineTranslationProps {
  text: string;
  translatedText?: string;
  targetLang?: string;
  confidence?: number;
  isLoading?: boolean;
  error?: string;
  onTranslate?: (text: string) => void;
}

export function InlineTranslation({
  text,
  translatedText,
  confidence,
  isLoading = false,
  error,
  onTranslate,
}: InlineTranslationProps) {
  const [showTranslation, setShowTranslation] = useState(!!translatedText);
  const [isHovered, setIsHovered] = useState(false);

  const handleTranslate = () => {
    if (onTranslate) {
      onTranslate(text);
    }
  };

  const toggleView = () => {
    setShowTranslation(!showTranslation);
  };

  return (
    <div
      className="inline-translation"
      data-testid="inline-translation"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div className="inline-translation-content">
        {isLoading ? (
          <Spin size="small" role="progressbar" />
        ) : showTranslation && translatedText ? (
          <span className="translated-text">{translatedText}</span>
        ) : (
          <span className="original-text">{text}</span>
        )}

        {confidence !== undefined && showTranslation && (
          <span
            className="confidence-badge"
            style={{
              marginLeft: '8px',
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
      </div>

      {error && <div className="inline-translation-error">{error}</div>}

      <div className={`inline-translation-actions ${isHovered ? 'visible' : ''}`}>
        {!translatedText && !isLoading && (
          <Tooltip content="Translate">
            <Button
              type="text"
              size="mini"
              icon={<IconTranslate />}
              onClick={handleTranslate}
              aria-label="Translate"
            />
          </Tooltip>
        )}

        {translatedText && !isLoading && (
          <Tooltip content={showTranslation ? 'Show original' : 'Show translation'}>
            <Button
              type="text"
              size="mini"
              icon={<IconSwap />}
              onClick={toggleView}
              aria-label={showTranslation ? 'Show original' : 'Show translation'}
            />
          </Tooltip>
        )}
      </div>
    </div>
  );
}
