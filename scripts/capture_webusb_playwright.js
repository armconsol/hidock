// Playwright script to launch Chrome with WebUSB monitor pre-loaded
// Usage: node scripts/capture_webusb_playwright.js

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

async function captureWebUSB() {
  console.log('🚀 Launching Chrome with WebUSB monitor...\n');

  // Read the WebUSB monitor script
  const monitorScript = fs.readFileSync(
    path.join(__dirname, 'webusb_monitor.js'),
    'utf8'
  );

  // Launch browser
  const browser = await chromium.launch({
    headless: false, // Show browser window
    devtools: true,  // Open DevTools automatically
    args: [
      '--enable-usb-user-gesture', // Allow USB without user gesture
    ],
  });

  const context = await browser.newContext({
    permissions: ['usb'], // Grant USB permission
  });

  const page = await context.newPage();

  // Navigate to HiNotes
  console.log('📡 Navigating to HiNotes webapp...');
  await page.goto('https://hinotes.hidock.com');

  // Wait for page to load
  await page.waitForLoadState('networkidle');

  console.log('✅ Page loaded');
  console.log('💉 Injecting WebUSB monitor script...\n');

  // Inject the monitor script
  await page.evaluate(monitorScript);

  console.log('✅ WebUSB monitor installed!');
  console.log('\n' + '='.repeat(60));
  console.log('INSTRUCTIONS:');
  console.log('='.repeat(60));
  console.log('1. The browser is now open with DevTools');
  console.log('2. Go to the Console tab in DevTools');
  console.log('3. You should see: "✓ HiDoc P1 USB Monitor Installed!"');
  console.log('4. REFRESH THE PAGE (Cmd+R) to capture from start');
  console.log('5. Perform device operations:');
  console.log('   - List files');
  console.log('   - Start recording');
  console.log('   - Stop recording');
  console.log('   - Play audio');
  console.log('   - Transfer files');
  console.log('   - Delete files');
  console.log('6. When done, run in Console:');
  console.log('   window.exportHidocLog()');
  console.log('7. Paste the output to a file');
  console.log('8. Press Ctrl+C here to close the browser');
  console.log('='.repeat(60) + '\n');

  // Keep the browser open
  console.log('⏳ Browser will stay open. Press Ctrl+C to close.\n');

  // Wait indefinitely (until user closes or Ctrl+C)
  await new Promise(() => {});
}

captureWebUSB().catch(console.error);
