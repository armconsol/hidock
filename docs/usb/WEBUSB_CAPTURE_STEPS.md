# WebUSB Capture - Quick Steps

**Chrome is launching with USB debug logging enabled!**

## What To Do Next:

### 1. Open Chrome DevTools
- Press `Cmd + Option + I` (or F12)
- Click on the **Console** tab

### 2. Copy and Paste the WebUSB Monitor Script

Copy the entire content from: `scripts/webusb_monitor.js`

```bash
# In Terminal, copy script to clipboard:
cat scripts/webusb_monitor.js | pbcopy
```

Then paste into Chrome DevTools Console and press Enter.

**You should see:** `✓ HiDoc P1 USB Monitor Installed!` in green text

### 3. Refresh the Page
Press `Cmd + R` to reload HiNotes webapp with the monitor active

### 4. Connect HiDoc P1 Device
- Plug in your HiDoc P1 via USB
- When HiNotes prompts for USB device access, click **Allow**
- The monitor will capture all USB operations

### 5. Perform These Operations (watch Console output):
1. **Device Detection** - Just connecting should trigger enumeration
2. **Recording** - Press record button, speak for 5-10 seconds, press stop
3. **Playback** - Press play button to hear recording
4. **File Transfer** - Sync/transfer files if available in UI
5. **Settings** - Change any device settings if available

### 6. Export Captured Protocol Data

In the Console, run:
```javascript
exportHidocLog()
```

This will copy the captured protocol log to your clipboard.

### 7. Save the Captured Data

Create a new file with the captured data:
```bash
# Paste from clipboard into new file:
pbpaste > usb_captures/hidoc_webusb_$(date +%Y%m%d_%H%M%S).json
```

## Expected Console Output Examples:

```
[USB] requestDevice called
[USB] Device granted: {vendorId: "0x10d6", productId: "0xb00e", ...}
[USB] open()
[USB] claimInterface: 0
[USB] controlTransferOut: {requestType: "vendor", ...}
  → Data: [0x01, 0x02, 0x03...]
[USB] controlTransferIn: {requestType: "vendor", ...}
  ← Response: {status: "ok", data: [0xAA, 0xBB...]}
[USB] transferOut: endpoint: 1
  → Data: [audio data...]
[USB] transferIn: endpoint: 2
  ← Received: [response data...]
```

## Troubleshooting:

**"WebUSB not available"**: Enable at `chrome://flags/#enable-experimental-web-platform-features`

**Device not detected**: 
- Check USB connection
- Try different USB port
- Verify device shows in System Profiler (macOS) with VID 0x10d6 or 0x1395

**No USB prompts**: Refresh page after pasting monitor script

## What We're Looking For:

- **Control commands** for record/stop/play
- **Audio data transfer** formats and endpoints
- **Device configuration** sequences
- **File transfer** protocols
- **Status polling** patterns

---

**Created:** 2026-08-18  
**For detailed documentation, see:** USB_CAPTURE_SESSION.md
