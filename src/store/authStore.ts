import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export type AuthProvider = 'google' | 'apple' | 'email';

export interface User {
  id: string;
  email: string;
  name?: string;
  avatar?: string;
  provider: AuthProvider;
}

interface AuthResult {
  user: {
    id: string;
    email: string;
    name: string;
  };
  token: string;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  loginWithEmail: (email: string, password: string) => Promise<void>;
  loginWithOAuth: (provider: 'google' | 'apple') => Promise<void>;
  logout: () => Promise<void>;
  clearError: () => void;
  setUser: (user: User | null) => void;
  hydrateAuth: () => Promise<void>;
  refreshToken: () => Promise<void>;
}

// Token storage keys
const TOKEN_KEY = 'hidoc_auth_token';
const TOKEN_EXPIRY_KEY = 'hidoc_auth_token_expiry';
const TOKEN_REFRESH_INTERVAL = 24 * 60 * 60 * 1000; // 24 hours

export const useAuthStore = create<AuthState>()((set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      loginWithEmail: async (email: string, password: string) => {
        set({ isLoading: true, error: null });
        try {
          const result = await invoke<AuthResult>('authenticate_with_credentials', {
            email,
            password,
          });

          const user: User = {
            id: result.user.id,
            email: result.user.email,
            name: result.user.name,
            provider: 'email',
          };

          // Store token in localStorage
          localStorage.setItem(TOKEN_KEY, result.token);
          localStorage.setItem(
            TOKEN_EXPIRY_KEY,
            String(Date.now() + TOKEN_REFRESH_INTERVAL)
          );

          set({
            user,
            token: result.token,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            error: errorMessage,
            isLoading: false,
            user: null,
            token: null,
            isAuthenticated: false,
          });
          throw error;
        }
      },

      loginWithOAuth: async (provider: 'google' | 'apple') => {
        set({ isLoading: true, error: null });
        try {
          const commandName = provider === 'google'
            ? 'authenticate_google'
            : 'authenticate_apple';

          const token = await invoke<string>(commandName);

          // For OAuth, we'll need to fetch user info separately
          // For now, create a basic user object
          const user: User = {
            id: token.substring(0, 16), // Temporary ID from token
            email: `user@${provider}.com`, // Placeholder - should be fetched from backend
            name: `${provider} User`, // Placeholder - should be fetched from backend
            provider,
          };

          // Store token in localStorage
          localStorage.setItem(TOKEN_KEY, token);
          localStorage.setItem(
            TOKEN_EXPIRY_KEY,
            String(Date.now() + TOKEN_REFRESH_INTERVAL)
          );

          set({
            user,
            token,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            error: errorMessage,
            isLoading: false,
            user: null,
            token: null,
            isAuthenticated: false,
          });
          throw error;
        }
      },

      logout: async () => {
        try {
          // Clear token from localStorage
          localStorage.removeItem(TOKEN_KEY);
          localStorage.removeItem(TOKEN_EXPIRY_KEY);

          set({
            user: null,
            token: null,
            isAuthenticated: false,
            error: null,
          });
        } catch (error) {
          console.error('Logout error:', error);
          // Clear local state even if backend call fails
          localStorage.removeItem(TOKEN_KEY);
          localStorage.removeItem(TOKEN_EXPIRY_KEY);

          set({
            user: null,
            token: null,
            isAuthenticated: false,
            error: null,
          });
        }
      },

      clearError: () => set({ error: null }),

      setUser: (user) => set({ user, isAuthenticated: !!user }),

      hydrateAuth: async () => {
        const token = localStorage.getItem(TOKEN_KEY);
        const expiry = localStorage.getItem(TOKEN_EXPIRY_KEY);

        if (!token || !expiry) {
          return;
        }

        // Check if token is expired
        if (Date.now() > parseInt(expiry, 10)) {
          // Token expired, clear it
          localStorage.removeItem(TOKEN_KEY);
          localStorage.removeItem(TOKEN_EXPIRY_KEY);
          set({
            user: null,
            token: null,
            isAuthenticated: false,
          });
          return;
        }

        // Token exists and is valid, restore auth state
        const currentUser = get().user;
        if (currentUser && token) {
          set({
            token,
            isAuthenticated: true,
          });
        }
      },

      refreshToken: async () => {
        const { token, user } = get();

        if (!token || !user) {
          return;
        }

        const expiry = localStorage.getItem(TOKEN_EXPIRY_KEY);
        if (!expiry) {
          return;
        }

        // Check if token needs refresh (within 1 hour of expiry)
        const expiryTime = parseInt(expiry, 10);
        const needsRefresh = Date.now() > expiryTime - 60 * 60 * 1000;

        if (needsRefresh) {
          // For now, just extend the expiry
          // In a real implementation, you would call a refresh token endpoint
          localStorage.setItem(
            TOKEN_EXPIRY_KEY,
            String(Date.now() + TOKEN_REFRESH_INTERVAL)
          );
        }
      },
}));
