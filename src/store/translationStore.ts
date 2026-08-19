import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { TranslationResponse, TranslationSettings, DEFAULT_SETTINGS } from '../components/Translation/types';

interface TranslationState {
  isLoading: boolean;
  error: string | null;
  translatedText: string | null;
  confidence: number | null;
  settings: TranslationSettings;

  // Actions
  translateText: (text: string, sourceLang: string, targetLang: string) => Promise<void>;
  detectLanguage: (text: string) => Promise<string>;
  clearCache: () => Promise<void>;
  saveSettings: (settings: TranslationSettings) => void;
  clearError: () => void;
}

export const useTranslationStore = create<TranslationState>((set) => ({
  isLoading: false,
  error: null,
  translatedText: null,
  confidence: null,
  settings: DEFAULT_SETTINGS,

  translateText: async (text: string, sourceLang: string, targetLang: string) => {
    set({ isLoading: true, error: null });

    try {
      const response = await invoke<TranslationResponse>('translate_text', {
        text,
        sourceLang,
        targetLang,
      });

      set({
        translatedText: response.translatedText,
        confidence: response.confidence ?? null,
        isLoading: false,
        error: null,
      });
    } catch (error) {
      set({
        isLoading: false,
        error: error instanceof Error ? error.message : 'Translation failed',
        translatedText: null,
        confidence: null,
      });
    }
  },

  detectLanguage: async (text: string) => {
    try {
      const language = await invoke<string>('detect_language', { text });
      return language;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Language detection failed',
      });
      throw error;
    }
  },

  clearCache: async () => {
    try {
      await invoke('clear_translation_cache');
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to clear cache',
      });
    }
  },

  saveSettings: (settings: TranslationSettings) => {
    // Store in localStorage
    localStorage.setItem('translationSettings', JSON.stringify(settings));
    set({ settings });
  },

  clearError: () => {
    set({ error: null });
  },
}));

// Load settings from localStorage on initialization
const storedSettings = localStorage.getItem('translationSettings');
if (storedSettings) {
  try {
    const settings = JSON.parse(storedSettings) as TranslationSettings;
    useTranslationStore.setState({ settings });
  } catch (error) {
    console.error('Failed to load translation settings:', error);
  }
}
