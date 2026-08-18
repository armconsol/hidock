import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type AuthProvider = 'google' | 'apple' | 'email';

export interface User {
  id: string;
  email: string;
  name?: string;
  avatar?: string;
  provider: AuthProvider;
}

export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  loginWithEmail: (email: string, password: string) => Promise<void>;
  loginWithOAuth: (provider: 'google' | 'apple') => Promise<void>;
  logout: () => void;
  clearError: () => void;
  setUser: (user: User | null) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      loginWithEmail: async (email: string, _password: string) => {
        set({ isLoading: true, error: null });
        try {
          // TODO: Implement actual email/password authentication
          // This is a placeholder implementation
          await new Promise((resolve) => setTimeout(resolve, 1000));

          const user: User = {
            id: '1',
            email,
            name: email.split('@')[0],
            provider: 'email',
          };

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Login failed',
            isLoading: false,
          });
        }
      },

      loginWithOAuth: async (provider: 'google' | 'apple') => {
        set({ isLoading: true, error: null });
        try {
          // TODO: Implement actual OAuth authentication
          // This is a placeholder implementation
          await new Promise((resolve) => setTimeout(resolve, 1000));

          const user: User = {
            id: '1',
            email: `user@${provider}.com`,
            name: `${provider} User`,
            provider,
          };

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'OAuth login failed',
            isLoading: false,
          });
        }
      },

      logout: () => {
        set({
          user: null,
          isAuthenticated: false,
          error: null,
        });
      },

      clearError: () => set({ error: null }),

      setUser: (user) => set({ user, isAuthenticated: !!user }),
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);
