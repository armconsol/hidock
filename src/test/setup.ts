import { expect, afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom';
import '@arco-design/web-react/dist/css/arco.css';

// Extend Vitest's expect with Testing Library matchers
expect.extend(matchers);

// Mock window.matchMedia for Arco Design components
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock navigator.clipboard for copy tests
Object.defineProperty(navigator, 'clipboard', {
  writable: true,
  configurable: true,
  value: {
    writeText: vi.fn().mockResolvedValue(undefined),
    readText: vi.fn().mockResolvedValue(''),
  },
});

/**
 * Mock Arco Design Message component
 *
 * Arco Design's Message component uses the deprecated ReactDOM.render() API
 * which is not available in React 19. This mock provides a test-compatible
 * implementation that captures Message calls without triggering DOM rendering errors.
 */
vi.mock('@arco-design/web-react', async () => {
  const actual = await vi.importActual('@arco-design/web-react');

  // Create mock Message methods that return a cleanup function
  const createMessageMethod = (type: string) => vi.fn((content: string) => {
    return {
      close: vi.fn(),
    };
  });

  return {
    ...actual,
    Message: {
      success: createMessageMethod('success'),
      error: createMessageMethod('error'),
      warning: createMessageMethod('warning'),
      info: createMessageMethod('info'),
      loading: createMessageMethod('loading'),
      clear: vi.fn(),
    },
  };
});

// Cleanup after each test
afterEach(() => {
  cleanup();
});
