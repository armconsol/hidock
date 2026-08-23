# Debugging Blank Page Issue

## Problem
After fixing the OAuth configuration issue and router integration, the app displays a blank white page.

## Troubleshooting Steps

### Step 1: Test Basic React Rendering ✅ DONE
Created minimal `main.tsx` with inline component to test if React renders at all.

**Test file:** `src/main.tsx` (currently simplified)
**Expected:** "HiNotes Desktop - Troubleshooting Mode" message
**If this works:** React is fine, issue is with router or components
**If this fails:** Deeper Tauri webview issue

### Step 2: Add Router Without Components
If Step 1 works, next test the router with a simple inline component:

```tsx
import { createBrowserRouter, RouterProvider } from 'react-router-dom';

const router = createBrowserRouter([
  {
    path: '/',
    element: <div><h1>Router Works!</h1></div>,
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <RouterProvider router={router} />
);
```

### Step 3: Add Arco Design CSS
If Step 2 works, add back Arco Design:

```tsx
import '@arco-design/web-react/dist/css/arco.css';
```

### Step 4: Add One Component at a Time
Test each component individually:
1. HomePage
2. AppLayout
3. Other pages

## Possible Causes

### 1. Missing CSS Files
- Arco Design CSS not bundled
- theme.css missing or broken

### 2. Component Import Errors
- Circular dependencies
- Missing exports
- TypeScript compilation errors silently failing

### 3. Store Initialization Issues
- Zustand stores throwing errors
- Missing initial state

### 4. Hook Errors
- useAuthLifecycle failing
- Other custom hooks throwing

### 5. Tauri Webview Issues
- CSP (Content Security Policy) blocking resources
- File protocol issues with CSS/assets

## Quick Fixes to Try

### Fix 1: Check Browser Console
In development mode:
```bash
npm run tauri dev
```
Then open DevTools (Cmd+Option+I on Mac) and check console for errors.

### Fix 2: Check Tauri CSP Configuration
File: `src-tauri/tauri.conf.json`
Look for `security.csp` and ensure it's not too restrictive.

### Fix 3: Verify Build Output
```bash
ls -la dist/
cat dist/index.html
```
Ensure assets are referenced correctly.

### Fix 4: Test with Dev Server
```bash
npm run dev
```
Open http://localhost:1420 in a regular browser to see console errors.

## Current Status

**Testing:** Minimal React component (no router, no external CSS)
**Next:** If successful, add router back incrementally

## Resolution Path

Once we identify which step fails, we'll know exactly what's causing the blank page:

- **Step 1 fails:** Tauri webview configuration issue
- **Step 2 fails:** Router setup problem
- **Step 3 fails:** CSS loading issue
- **Step 4 fails:** Specific component error

Then we can apply the appropriate fix.
