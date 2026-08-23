import { expect, afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';
import 'antd/dist/reset.css';

// Extend Vitest's expect with Testing Library matchers
expect.extend(matchers);

// Mock window.matchMedia for Ant Design components
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
 * Mock Ant Design message component
 *
 * Ant Design's message component may use APIs that need mocking in the test environment.
 * This mock provides a test-compatible implementation that captures message calls
 * without triggering DOM rendering errors.
 */
vi.mock('antd', async () => {
  const actual = await vi.importActual('antd');

  // Create mock message methods that return a cleanup function
  const createMessageMethod = (_type: string) => vi.fn((_content: string) => {
    return {
      close: vi.fn(),
    };
  });

  return {
    ...actual,
    message: {
      success: createMessageMethod('success'),
      error: createMessageMethod('error'),
      warning: createMessageMethod('warning'),
      info: createMessageMethod('info'),
      loading: createMessageMethod('loading'),
      destroy: vi.fn(),
    },
  };
});

// Cleanup after each test
afterEach(() => {
  cleanup();
});
