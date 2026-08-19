import { useEffect } from 'react';
import { useAuthStore } from '../store/authStore';

/**
 * Hook to manage authentication lifecycle events
 * - Hydrates auth state on mount
 * - Sets up token refresh on app resume
 * - Periodic token refresh check
 */
export function useAuthLifecycle() {
  const hydrateAuth = useAuthStore((state) => state.hydrateAuth);
  const refreshToken = useAuthStore((state) => state.refreshToken);

  useEffect(() => {
    // Hydrate auth state from localStorage on mount
    hydrateAuth();

    // Set up periodic token refresh check (every 30 minutes)
    const refreshInterval = setInterval(() => {
      refreshToken();
    }, 30 * 60 * 1000);

    // Handle app resume/visibility change
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        // App became visible, refresh token if needed
        refreshToken();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    // Cleanup
    return () => {
      clearInterval(refreshInterval);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [hydrateAuth, refreshToken]);
}
