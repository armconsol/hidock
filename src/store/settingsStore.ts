import { create } from 'zustand';

type Theme = 'light' | 'dark';

interface SettingsState {
  theme: Theme;

  // Actions
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

export const useSettingsStore = create<SettingsState>()((set) => ({
  theme: 'light',

  setTheme: (theme) => set({ theme }),

  toggleTheme: () =>
    set((state) => ({
      theme: state.theme === 'light' ? 'dark' : 'light',
    })),
}));
