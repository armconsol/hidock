# Quick USB Protocol Capture Guide

## Chrome is Now Open!

A Chrome window should be open with:
- **HiNotes webapp** loaded: https://hinotes.hidock.com
- **DevTools** open (if not, press `Cmd + Option + I`)

## Step 1: Install the Monitor (30 seconds)

1. **Go to the Console tab** in DevTools
2. **Copy this entire script:**

```bash
cat ~/Documents/GitHub/hinotes-desktop-new/scripts/webusb_monitor.js | pbcopy
```

3. **Paste into Console** and press Enter
4. You should see: **"✓ HiDoc P1 USB Monitor Installed!"** in green

## Step 2: Refresh the Page

**Press `Cmd + R`** to refresh the page

This ensures the monitor captures USB activity from the very beginning.

## Step 3: Perform Device Operations

With the HiDoc P1 connected, perform these operations **one at a time**:

### A. Device Connection
- The device should auto-connect
- Watch Console for USB messages

### B. List Files  
- Navigate to files/recordings section
- Watch for `transferIn`/`transferOut` messages

### C. Record Audio
- Press record button on HiDoc P1
- Record for 5-10 seconds
- Press stop
- Watch for command packets

### D. Play Audio
- Click to play a recording
- Watch for playback commands

### E. Transfer/Sync Files
- If there's a sync button, click it
- Watch for file transfer protocol

### F. Delete File
- Delete a recording
- Watch for delete commands

## Step 4: Export Captured Data

When you're done, in the Console type:

```javascript
window.exportHidocLog()
```

This copies the entire protocol log to your clipboard!

## Step 5: Save the Log

Paste into a file:

```bash
# The clipboard now has the JSON
# Just paste it:
pbpaste > ~/Documents/GitHub/hinotes-desktop-new/usb_captures/protocol_$(date +%Y%m%d_%H%M%S).json
```

## What to Look For

In the Console, you'll see color-coded messages:

- **🔵 Blue (transferOut):** Commands sent TO device
- **🟢 Green (transferIn):** Responses FROM device  
- **🟣 Purple (open/close):** Device connection
- **🟠 Orange (claimInterface):** Interface selection
- **Hex dumps:** Raw data in hexadecimal

### Example Output:
```
[USB] transferOut: endpoint 1
  → Data: {bytes: 16, data: [72, 68, 79, 67, 1, 0, ...], hex: "48 44 4f 43 01 00 ..."}
[USB] transferIn: endpoint 1 length: 256
  ← Received: {status: "ok", bytesRead: 64, data: [...], hex: "..."}
```

## Troubleshooting

**No USB messages appearing?**
- Make sure you refreshed AFTER installing the monitor
- Try disconnecting/reconnecting the HiDoc P1
- Check that Chrome has USB permission

**"USB API not available"?**
- WebUSB only works in Chrome/Edge
- Must be on HTTPS (hinotes.hidock.com is HTTPS ✓)

**Device not connecting?**
- Close any other apps using the device
- Check System Settings → Privacy & Security → USB

## After Capture

Once you have the protocol log:

1. **Analyze the patterns** - Look for command structures
2. **Document commands** - Create protocol specification
3. **Implement in Rust** - Update `src-tauri/src/usb/protocol.rs`
4. **Test with rusb** - Send same commands from Rust app

---

**Ready?** Go to Chrome Console and paste the monitor script! 🚀

The script is already copied to your clipboard if you ran:
```bash
cat ~/Documents/GitHub/hinotes-desktop-new/scripts/webusb_monitor.js | pbcopy
```
