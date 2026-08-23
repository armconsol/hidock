import { Select } from 'antd';
import { SUPPORTED_LANGUAGES } from './types';

interface LanguageSelectorProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  showSearch?: boolean;
  showNativeName?: boolean;
  allowAutoDetect?: boolean;
}

export function LanguageSelector({
  value,
  onChange,
  disabled = false,
  showSearch = false,
  showNativeName = false,
  allowAutoDetect = false,
}: LanguageSelectorProps) {

  const options = [
    ...(allowAutoDetect
      ? [{ value: 'auto', label: 'Auto-detect', flag: '🔍' }]
      : []),
    ...SUPPORTED_LANGUAGES.map((lang) => ({
      value: lang.code,
      label: showNativeName ? `${lang.name} (${lang.nativeName})` : lang.name,
      flag: lang.flag,
    })),
  ];

  return (
    <Select
      value={value}
      onChange={onChange}
      disabled={disabled}
      showSearch={showSearch}
      filterOption={(inputValue, option) => {
        const optLabel = options.find(o => o.value === (option as any).value)?.label || '';
        return optLabel.toLowerCase().includes(inputValue.toLowerCase());
      }}
      style={{ width: '100%' }}
    >
      {options.map((option) => (
        <Select.Option key={option.value} value={option.value}>
          <span role="img" aria-label={`${option.label} flag`}>
            {option.flag}
          </span>{' '}
          {option.label}
        </Select.Option>
      ))}
    </Select>
  );
}
