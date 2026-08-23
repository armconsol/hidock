import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Ant Design works correctly with default Vite configuration
  build: {
    commonjsOptions: {
      transformMixedEsModules: true,
    },
  },

  // Resolve aliases for test mocks
  resolve: {
    alias: {
      '@tauri-apps/plugin-dialog': new URL('./src/test/mocks/tauri-dialog.ts', import.meta.url).pathname,
    },
  },

  // Test configuration
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    // Default 5000ms is too tight for userEvent.type() interactions under a
    // loaded/shared CI runner (multiple parallel jobs on the same host) --
    // these tests run in <1s locally but can exceed 5000ms in CI, causing
    // flaky failures unrelated to app behavior.
    testTimeout: 15000,
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
