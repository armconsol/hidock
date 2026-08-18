import { ConfigProvider } from '@arco-design/web-react';
import { useSettingsStore } from '../store/settingsStore';
import { ReactNode, useEffect } from 'react';

interface ThemeProviderProps {
  children: ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const theme = useSettingsStore((state) => state.theme);

  // Apply theme to document
  useEffect(() => {
    document.documentElement.setAttribute('arco-theme', theme);
  }, [theme]);

  return (
    <ConfigProvider
      theme={{
        mode: theme,
      }}
    >
      {children}
    </ConfigProvider>
  );
}
