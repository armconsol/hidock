# HiDoc P1 USB Protocol Capture Session Guide

**Created:** 2026-08-18  
**Method:** WebUSB monitoring via Chrome DevTools  
**Target Device:** HiDoc P1 Audio Transcription Device

---

## Session Overview

This guide walks through a complete USB protocol capture session for the HiDoc P1 device. The goal is to capture and analyze the USB communication protocol used by the official HiNotes web application to control the device.

**Why WebUSB Method?**
- Captures application-level protocol (already parsed)
- No root/admin privileges required
- Human-readable command sequences
- Easy to correlate actions with packets
- Faster than raw USB packet analysis

---

## Device Identification

The HiDoc P1 exposes **two USB interfaces**:

### Audio Interface (Actions Semiconductor)
```
Vendor ID (VID):  0x10d6
Product ID (PID): 0xb00e
Product Name:     HiDock_P1
Manufacturer:     HiDock
Serial Number:    ACTIONS-BOS-002
Speed:            USB 2.0 High Speed (480 Mb/s)
```
**Purpose:** Audio capture/playback

### Control Interface (Solid State System)
```
Vendor ID (VID):  0x1395
Product ID (PID): 0x005d
Product Name:     HiDock P1
Manufacturer:     Solid State System Co.,Ltd.
Speed:            USB 1.1 Full Speed (12 Mb/s)
```
**Purpose:** Device control, file transfer, configuration

---

## Prerequisites

### Required Software
- [x] **Google Chrome** or **Microsoft Edge** (WebUSB support)
- [x] **HiDoc P1 device** (connected via USB)
- [x] **Internet connection** (to access HiNotes webapp)

### Required Files
- [x] **WebUSB Monitor Script:** `/Users/sarman/Documents/GitHub/hidoc/scripts/webusb_monitor.js`
- [x] **Capture Storage Directory:** `/Users/sarman/Documents/GitHub/hidoc/usb_captures/`

### Browser Configuration
Ensure WebUSB is enabled:
1. Open `chrome://flags/#enable-experimental-web-platform-features`
2. Set to **Enabled**
3. Restart Chrome

---

## Session Start Procedure

### Step 1: Launch Chrome with Verbose USB Logging

Open Terminal and run:
```bash
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --enable-logging \
  --v=1 \
  --user-data-dir=/tmp/chrome-usb-debug \
  https://hinotes.hidock.com
```

**What this does:**
- `--enable-logging`: Enables Chrome's internal logging
- `--v=1`: Sets verbosity level to 1 (basic USB events)
- `--user-data-dir=/tmp/chrome-usb-debug`: Uses temporary profile (clean state)
- Opens HiNotes web application directly

**Expected Result:** Chrome launches with a clean profile and navigates to HiNotes.

### Step 2: Install WebUSB Monitor Script

1. **Open Chrome DevTools:**
   - Press `Cmd + Option + I` (Mac)
   - Or press `F12` (Windows/Linux)
   - Or right-click → "Inspect"

2. **Navigate to Console tab**

3. **Load the monitor script:**
   ```bash
   # In a separate terminal, copy the script to clipboard:
   cat /Users/sarman/Documents/GitHub/hidoc/scripts/webusb_monitor.js | pbcopy
   ```

4. **Paste into Chrome Console** and press Enter

5. **Verify installation:**
   You should see:
   ```
   ✓ HiDoc P1 USB Monitor Installed!
   Now refresh the page or connect your device
   Protocol log will be saved in: window.hidocProtocolLog
   To export: copy(JSON.stringify(window.hidocProtocolLog, null, 2))
   ```

**Troubleshooting:**
- If you see errors, make sure you copied the entire script
- Clear console (`Cmd+K`) and try again
- Check for browser extensions that might interfere with USB

### Step 3: Refresh Page (Critical!)

**IMPORTANT:** Refresh the page AFTER installing the monitor to capture device connection from the start.

1. Press `Cmd+R` (Mac) or `F5` (Windows/Linux)
2. The monitor script will be lost (this is normal)
3. **Paste the script again** into the Console
4. Now the monitor is active and ready to capture

**Why refresh?** The device may have already connected. Refreshing forces a new connection sequence that the monitor can capture.

---

## Operations to Capture

Perform these operations **one at a time** and take notes on what you observe in the Console.

### Operation 1: Device Connection & Enumeration

**Action:** Ensure device is connected (or reconnect it)

**What to observe:**
- `[USB] requestDevice called` - Browser requests access
- `[USB] Device granted:` - Shows VID/PID/manufacturer
- `[USB] open()` - Device opened
- `[USB] selectConfiguration:` - Configuration selected (usually 1)
- `[USB] claimInterface:` - Interface claimed (watch which number)

**Expected VID/PID:**
- Audio: `0x10d6:0xb00e`
- Control: `0x1395:0x005d`

**Notes to record:**
- Which interface is claimed first?
- Are both interfaces claimed?
- What configuration value is used?

### Operation 2: Initial Device Query

**Action:** Wait for page to fully load

**What to observe:**
- Initial command packets (likely device info query)
- Response packets with device status
- Battery level query (if applicable)
- Firmware version query

**Look for:**
- Command packet structure (magic bytes, command ID)
- Response format (status codes, payload)
- Endpoint numbers used (IN/OUT)

### Operation 3: File List / Recording List

**Action:** Navigate to recordings/files section in HiNotes

**What to observe:**
- `[USB] transferOut:` - List command sent
- `[USB] transferIn:` - File list received
- Multiple transfers if list is paginated

**Key data to capture:**
- Command byte sequence
- How files are identified (IDs, names?)
- Timestamp format
- File size encoding

### Operation 4: Start Recording

**Action:** Press the **Record** button on the HiDoc P1 device

**What to observe:**
- Recording start command
- Acknowledgment response
- Periodic status updates (if any)

**Notes:**
- Is command sent from device to host, or host to device?
- Does web app poll for status, or does device push?

### Operation 5: Stop Recording

**Action:** Press **Stop** on the device

**What to observe:**
- Recording stop command
- Final status/metadata (duration, file size)
- File ID assignment

### Operation 6: File Transfer (Download)

**Action:** Click to download/sync a recording from device

**What to observe:**
- File request command (with file ID?)
- Data transfer packets (watch for chunking)
- Transfer completion confirmation

**Key questions:**
- How are files identified in transfer command?
- What's the chunk size?
- Is there a checksum/CRC?
- How is EOF signaled?

### Operation 7: Audio Playback

**Action:** Click to play a recording in HiNotes

**What to observe:**
- Does it transfer file first, or stream?
- If streaming, watch for audio packet structure
- Playback control commands (play, pause, stop)

### Operation 8: File Deletion

**Action:** Delete a recording from device

**What to observe:**
- Delete command structure
- How file is identified (ID, name?)
- Confirmation response
- Updated file list query

### Operation 9: Device Settings (if available)

**Action:** Change any device settings in HiNotes UI (if present)

**What to observe:**
- Settings read command (get current config)
- Settings write command
- Confirmation/validation response

---

## Exporting Captured Data

### During Capture Session

The monitor automatically saves to `localStorage` when you switch tabs. You can also export manually at any time.

### Method 1: Auto-Export Function (Recommended)

In Chrome Console:
```javascript
window.exportHidocLog()
```

This copies the entire log to clipboard. Then in Terminal:
```bash
# Save to timestamped file
pbpaste > /Users/sarman/Documents/GitHub/hidoc/usb_captures/protocol_log_$(date +%Y%m%d_%H%M%S).json
```

### Method 2: Manual JSON Export

In Chrome Console:
```javascript
copy(JSON.stringify(window.hidocProtocolLog, null, 2))
```

Then paste into a text editor and save to:
```
/Users/sarman/Documents/GitHub/hidoc/usb_captures/session_<description>.json
```

### Method 3: Save Console Output

1. Right-click in Console
2. Select "Save as..."
3. Save to `usb_captures/console_log_<timestamp>.txt`

**Best Practice:** Use all three methods for redundancy.

---

## Captured Data Structure

Each log entry contains:
```json
{
  "index": 0,
  "timestamp": 1724012345678,
  "time": "2026-08-18T10:30:45.678Z",
  "type": "transferOut",
  "endpoint": 1,
  "dataLength": 16,
  "data": [72, 68, 79, 67, 1, 0, 0, 0, 8, 0, 0, 0, 255, 255, 0, 0],
  "hex": "48 44 4f 43 01 00 00 00 08 00 00 00 ff ff 00 00"
}
```

### Log Entry Types

| Type | Description |
|------|-------------|
| `requestDevice` | Browser requests USB device access |
| `deviceGranted` | User grants permission, device info returned |
| `open` | Device opened for communication |
| `close` | Device closed |
| `selectConfiguration` | USB configuration selected |
| `claimInterface` | USB interface claimed |
| `controlTransferIn` | Control transfer (device → host) |
| `controlTransferOut` | Control transfer (host → device) |
| `transferIn` | Bulk/Interrupt transfer (device → host) |
| `transferOut` | Bulk/Interrupt transfer (host → device) |

---

## Analysis Guidelines

### Look for Protocol Patterns

#### 1. Magic Bytes / Header
- First 4 bytes often spell device name (e.g., "HDOC" = `48 44 4F 43`)
- Consistent across all commands
- Used to identify valid packets

#### 2. Command Byte
- Usually byte 4 or 5
- Changes based on operation:
  - `0x01` might be "list files"
  - `0x02` might be "transfer file"
  - `0x03` might be "delete file"
  - `0x10` might be "start recording"
  - `0x11` might be "stop recording"

#### 3. Sequence Number
- Often 2 bytes
- Increments with each command
- Response matches request sequence

#### 4. Length Field
- Usually 2 or 4 bytes
- Indicates payload size
- May be little-endian or big-endian (test both)

#### 5. Payload
- Variable length
- Contains command-specific data
- May include:
  - File IDs
  - Timestamps
  - Settings values
  - File data chunks

#### 6. Checksum / CRC
- Often last 2 or 4 bytes
- Validates packet integrity
- May be CRC16, CRC32, or simple sum

### Pattern Recognition Steps

1. **Group by operation** - Separate captures by action performed
2. **Compare similar operations** - Find what stays the same vs what changes
3. **Correlate with responses** - Match OUT packets with IN packets
4. **Look for ASCII strings** - File names, device names might be readable
5. **Test hypotheses** - Change one byte at a time in reconstructed packets

---

## Troubleshooting

### No USB Activity in Console

**Symptoms:** Monitor installed but no log entries appear

**Solutions:**
1. **Refresh page** after installing monitor
2. **Disconnect and reconnect** the device
3. **Check console filters** - Remove any active filters
4. **Verify WebUSB is enabled** - Check `chrome://flags`
5. **Try a different USB port** - Some ports may have issues
6. **Close other apps** using the device (especially native HiNotes app)

### "USB API not available" Error

**Symptoms:** `navigator.usb` is undefined

**Solutions:**
1. **Use Chrome or Edge** - Firefox/Safari don't support WebUSB
2. **Check HTTPS** - WebUSB requires HTTPS or localhost
3. **Enable flag** - `chrome://flags/#enable-experimental-web-platform-features`
4. **Update Chrome** - Ensure latest version

### Device Access Denied

**Symptoms:** Permission popup doesn't show, or access denied error

**Solutions:**
1. **System permissions** - Check macOS System Preferences → Security & Privacy
2. **Close competing apps** - Other apps may have exclusive access
3. **Unplug/replug device** - Reset USB connection
4. **Try different user profile** - `--user-data-dir=/tmp/chrome-test`
5. **Check Chrome permissions** - `chrome://settings/content/usbDevices`

### Partial Data Capture

**Symptoms:** Some operations logged, others missing

**Solutions:**
1. **Increase buffer size** - Large transfers may be truncated
2. **Slow down operations** - Wait between actions for log to flush
3. **Export frequently** - Don't rely on single export at end
4. **Check localStorage quota** - May fill up on long sessions

### Data Too Large

**Symptoms:** Export fails or truncates

**Solutions:**
1. **Export incrementally** - Save after each major operation
2. **Clear old entries** - `window.hidocProtocolLog = []` to reset
3. **Reduce data logging** - Modify script to log first 64 bytes only
4. **Use file save** instead of clipboard

---

## Expected Results

After a successful capture session, you should have:

### Captured Communication Sequences

- [x] **Device enumeration** (VID, PID, interfaces, endpoints)
- [x] **Configuration & interface claiming**
- [x] **Initial device query/handshake**
- [x] **File list command and response**
- [x] **Recording start/stop commands**
- [x] **File transfer protocol** (read file from device)
- [x] **File delete command**
- [x] **Audio playback commands** (if applicable)
- [x] **Device settings read/write** (if applicable)

### Protocol Information Discovered

- Command packet structure (header, command, payload, checksum)
- Response packet structure (status codes, data format)
- Endpoint usage (which operations use which endpoints)
- Transfer sizes (max packet size for commands/data)
- Timing (delays between commands, timeout values)
- Error handling (what happens on failure)

---

## Post-Capture Workflow

### 1. Organize Captured Files

Create timestamped directory:
```bash
cd /Users/sarman/Documents/GitHub/hidoc/usb_captures
mkdir session_$(date +%Y%m%d_%H%M%S)
mv protocol_log_*.json session_*/
```

### 2. Initial Analysis

Open the JSON file and:
- Count total operations
- Identify unique command bytes
- Group by endpoint number
- Look for ASCII strings in data

### 3. Create Command Reference

Document discovered commands in structured format:
```bash
# Example: commands_reference.md
# Command 0x01: List Files
# Endpoint: 1 OUT → 1 IN
# Request:  48 44 4f 43 01 00 00 00 ...
# Response: 48 44 4f 43 00 01 00 00 ...
```

### 4. Update Protocol Documentation

Add findings to:
- `USB_PROTOCOL_ANALYSIS.md` - Detailed analysis
- `USB_PROTOCOL_SPEC.md` - Formal specification
- `HIDOC_P1_DEVICE_INFO.md` - Device details

### 5. Implement in Rust

Update Rust code with discovered protocol:
- `src-tauri/src/usb/protocol.rs` - Packet structures
- `src-tauri/src/usb/commands.rs` - Command functions
- `src-tauri/src/usb/device.rs` - Device communication

### 6. Create Test Cases

Use captured packets as test data:
```rust
#[test]
fn test_parse_file_list_response() {
    let captured_data = vec![0x48, 0x44, 0x4F, 0x43, ...];
    let result = parse_file_list(&captured_data);
    assert!(result.is_ok());
}
```

---

## Tips for Successful Capture

### Before Starting

- [x] **Charge the device** - Avoid disconnections mid-session
- [x] **Close other apps** - Prevent USB conflicts
- [x] **Test monitor first** - Verify it logs any USB activity
- [x] **Have device manuals ready** - Reference for button functions

### During Capture

- [ ] **Take detailed notes** - Document each action as you perform it
- [ ] **Repeat operations** - Capture each action 2-3 times to confirm patterns
- [ ] **Work methodically** - One operation at a time, wait for completion
- [ ] **Export frequently** - Save after each major operation group
- [ ] **Keep DevTools visible** - Watch for unexpected errors

### After Capture

- [ ] **Back up data immediately** - Copy to multiple locations
- [ ] **Review while fresh** - Analyze within 24 hours while you remember
- [ ] **Share findings** - Commit to git repo with descriptive messages
- [ ] **Document unknowns** - Note unclear patterns for future investigation

---

## Advanced: Modifying the Monitor Script

If you need to capture different data, edit `/Users/sarman/Documents/GitHub/hidoc/scripts/webusb_monitor.js`:

### Capture Full Transfers (Not Just 64 Bytes)
```javascript
// Find this line in transferIn hook:
data: Array.from(data).slice(0, 64),

// Change to:
data: Array.from(data), // Capture ALL bytes
```

### Add Custom Annotations
```javascript
// Add after logEntry() calls:
if (setup.request === 0x06) {
  console.log('%c  ⚠️ This looks like a descriptor request!', 'color: yellow');
}
```

### Filter Specific Commands
```javascript
// Skip logging certain commands:
if (dataArray[4] === 0xFF) {
  console.log('%c  [Skipped heartbeat packet]', 'color: gray');
  return originalTransferOut.call(this, endpointNumber, data);
}
```

---

## Chrome USB Logs (Alternative)

The `--enable-logging --v=1` flags also create log files:

**Location (macOS):**
```bash
/tmp/chrome-usb-debug/chrome_debug.log
```

**View USB-related entries:**
```bash
grep -i usb /tmp/chrome-usb-debug/chrome_debug.log
```

These logs contain lower-level USB events but are harder to correlate with application actions. Use as supplementary data only.

---

## File Naming Convention

Save captured files with descriptive names:

```
protocol_log_YYYYMMDD_HHMMSS.json         # Full session
session_list_files_001.json               # Specific operation
session_record_start_stop_002.json        # Related operations
session_file_transfer_large_003.json      # Notable characteristics
console_output_YYYYMMDD_HHMMSS.txt        # Console text backup
```

---

## Next Steps After Successful Capture

1. **Analyze the Protocol** - Use `USB_PROTOCOL_ANALYSIS.md` as template
2. **Document Command Structures** - Create formal specification
3. **Implement in Rust** - Use `rusb` to send captured commands
4. **Test with Real Device** - Verify Rust implementation works
5. **Build Desktop Features** - Integrate into Tauri application

---

## References

- **WebUSB API:** https://developer.mozilla.org/en-US/docs/Web/API/USB
- **HiDoc P1 Device Info:** `HIDOC_P1_DEVICE_INFO.md`
- **Capture Instructions:** `USB_CAPTURE_INSTRUCTIONS.md`
- **Protocol Analysis:** `USB_PROTOCOL_ANALYSIS.md`
- **rusb Documentation:** https://docs.rs/rusb/

---

## Session Checklist

Use this checklist during your capture session:

- [ ] Chrome launched with `--enable-logging --v=1`
- [ ] DevTools opened (Cmd+Option+I)
- [ ] Monitor script pasted into Console
- [ ] Success message displayed
- [ ] Page refreshed after monitor installation
- [ ] Monitor script pasted again after refresh
- [ ] Device connected and detected
- [ ] Device connection sequence captured
- [ ] File list operation performed and logged
- [ ] Recording start/stop performed and logged
- [ ] File transfer performed and logged
- [ ] File deletion performed and logged
- [ ] Additional operations performed as needed
- [ ] Data exported via `window.exportHidocLog()`
- [ ] JSON saved to `usb_captures/` directory
- [ ] Console log saved as backup
- [ ] Notes taken for each operation
- [ ] Data backed up to git repository

---

**Good luck with your capture session!** This is the most efficient way to reverse engineer the HiDoc P1 USB protocol. The WebUSB monitor gives you high-level protocol visibility without the complexity of raw USB packet analysis.

For questions or issues, refer to the troubleshooting section or consult the other USB documentation files in this repository.
