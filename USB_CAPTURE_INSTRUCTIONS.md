# HiDoc P1 WebUSB Protocol Capture - Step by Step

## Prerequisites
- HiDoc P1 device connected to Mac
- Google Chrome browser
- HiNotes webapp access: https://hinotes.hidock.com

## Step 1: Install Monitor Script

1. **Open Chrome** and navigate to: https://hinotes.hidock.com
2. **Open DevTools:** Press `Cmd + Option + I` (or View → Developer → Developer Tools)
3. **Go to Console tab**
4. **Copy the entire monitor script:**
   ```bash
   cat /Users/sarman/Documents/GitHub/hinotes-desktop-new/scripts/webusb_monitor.js | pbcopy
   ```
   Or open: `/Users/sarman/Documents/GitHub/hinotes-desktop-new/scripts/webusb_monitor.js`
   
5. **Paste into Console** and press Enter
6. You should see: **"✓ HiDoc P1 USB Monitor Installed!"** in green

## Step 2: Refresh Page (Important!)

1. **Refresh the HiNotes webapp** (Cmd+R) to capture device connection from start
2. The monitor will now log ALL USB activity

## Step 3: Perform Operations

With the monitor active, perform these operations **one at a time** and note what you're doing:

### A. Device Connection
- The device should connect automatically
- Watch Console for connection messages

### B. List Files
- Navigate to the files/recordings section
- Observe file listing commands in Console

### C. Start Recording
- Press the record button on HiDoc P1 device
- Watch for recording start commands

### D. Stop Recording  
- Press stop button
- Watch for recording stop commands

### E. Play Recording
- Click to play a recorded file
- Watch for playback commands

### F. Transfer File
- If there's a sync/transfer button, click it
- Watch for file transfer protocol

### G. Delete File
- Delete a recording
- Watch for delete commands

## Step 4: Export Captured Data

### Option A: Auto-Export (Easiest)
```javascript
window.exportHidocLog()
```
This copies the log to clipboard. Paste into a file:
```bash
pbpaste > /Users/sarman/Documents/GitHub/hinotes-desktop-new/usb_captures/protocol_log_$(date +%Y%m%d_%H%M%S).json
```

### Option B: Manual Export
```javascript
copy(JSON.stringify(window.hidocProtocolLog, null, 2))
```
Then paste into a text file.

### Option C: Console Save
Right-click in Console → "Save as..." → Save the entire console log

## Step 5: Analyze Captured Data

The log contains entries like:
```json
{
  "index": 0,
  "timestamp": 1724012345678,
  "time": "2026-08-18T10:30:45.678Z",
  "type": "transferOut",
  "endpoint": 1,
  "dataLength": 16,
  "data": [72, 68, 79, 67, 1, 0, 0, 0, ...],
  "hex": "48 44 4f 43 01 00 00 00 ..."
}
```

### Look for patterns:
1. **Magic bytes** - First few bytes that repeat (e.g., "HDOC" = 0x48 0x44 0x4F 0x43)
2. **Command IDs** - Byte that changes based on operation
3. **Length fields** - Bytes indicating payload size
4. **Checksums** - Last bytes for validation
5. **Endpoint numbers** - Which endpoint is used for what

## Step 6: Document Findings

Create file: `USB_PROTOCOL_SPEC.md`

```markdown
# HiDoc P1 USB Protocol Specification

## Discovered Commands

### Command 0x01: List Files
**Endpoint:** 1 (OUT)
**Packet Structure:**
- Bytes 0-3: Magic "HDOC" (0x48 0x44 0x4F 0x43)
- Byte 4: Command ID (0x01)
- Bytes 5-6: Sequence number
- Bytes 7-8: Length (0x0000)

**Response Endpoint:** 1 (IN)
**Response Structure:**
- Bytes 0-3: Magic "HDOC"
- Byte 4: Status (0x00 = success)
- Bytes 5-6: Sequence (matching request)
- Bytes 7-8: Payload length
- Bytes 9+: File list data

### Command 0x02: Transfer File
...
```

## Troubleshooting

### No USB activity shown:
- Make sure you refreshed the page AFTER installing the monitor
- Check if Chrome DevTools is filtering console output (remove filters)
- Try disconnecting and reconnecting the device

### "USB API not available":
- WebUSB only works in Chrome/Edge
- Must be on HTTPS or localhost
- Check chrome://flags/#enable-experimental-web-platform-features

### Device access denied:
- Close any other apps using the device
- Chrome might need permission to access USB devices
- System Preferences → Security & Privacy → USB access

## Expected Results

You should capture:
- **Device enumeration** (VID/PID, interfaces, endpoints)
- **Configuration selection** (which config/interface)
- **File listing commands** (structure, response format)
- **Recording control** (start/stop commands)
- **File transfer protocol** (chunked data, metadata)
- **Delete commands** (file ID format)

## Next Steps After Capture

1. **Analyze the JSON log** - Look for patterns
2. **Document command structures** - Create protocol spec
3. **Implement in Rust** - Update `src-tauri/src/usb/protocol.rs`
4. **Test with rusb** - Send same commands from Rust
5. **Verify responses** - Match webapp behavior

## Tips

- **Take notes** as you perform operations (which button, what happened)
- **Repeat operations** to confirm patterns
- **Try edge cases** (empty file list, delete all, etc.)
- **Export frequently** - Don't lose captured data
- **Keep Chrome DevTools open** during entire session

## Files to Create After Analysis

1. `USB_PROTOCOL_SPEC.md` - Complete protocol documentation
2. `usb_captures/commands_reference.json` - Command/response examples
3. Update `src-tauri/src/usb/protocol.rs` - Rust implementation
4. Add tests with captured packet data

---

**Good luck with the capture!** 🎯

This is the **easiest way** to reverse engineer the protocol - much better than raw USB packet capture!
