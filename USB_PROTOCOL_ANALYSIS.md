# HiDoc P1 USB Protocol Analysis

**Status:** WebUSB Monitoring Tools Ready - Awaiting Manual Capture  
**Last Updated:** 2026-08-18 (15:30 CDT)

---

## Executive Summary

### What We Know (Confirmed ✅)
1. **Device has dual USB interfaces:**
   - **Audio Interface:** VID=0x10d6, PID=0xb00e (Actions Semiconductor) - Standard USB Audio Class
   - **Control Interface:** VID=0x1395, PID=0x005d (Solid State System) - Custom protocol
2. **Audio specifications:** 48kHz, 16-bit PCM, mono input, stereo output
3. **WebUSB access:** HiNotes webapp uses Chrome WebUSB API (exclusive access)
4. **USB speed:** 480 Mb/s (USB 2.0 High Speed) for audio, 12 Mb/s (USB 1.1 Full Speed) for control

### What We've Built (Ready ✅)
1. **Complete WebUSB monitoring toolkit** - Intercepts all USB API calls in Chrome
2. **Three capture guides** - Quick start, detailed walkthrough, and instructions
3. **Data logging infrastructure** - Timestamps, hex dumps, auto-save, export helpers

### What We Need (Blocked ⏳)
1. **Manual capture session** - Requires physical HiDoc P1 device access (30-60 min)
2. **Protocol analysis** - After capture data available (2-4 hours)
3. **Rust implementation** - After protocol documented (4-8 hours)
4. **Integration & testing** - After implementation complete (4-6 hours)

### Critical Path
```
Physical Device Access → Capture Session (1 hr) → Protocol Analysis (4 hr) → 
Rust Implementation (8 hr) → Testing (6 hr) = ~19 hours total
```

**Current Blocker:** Manual capture session requires user with physical HiDoc P1 device access.

---

## Device Specifications (Confirmed)

### Hardware Configuration
- **Dual USB Interfaces:**
  - Audio Interface: VID=0x10d6, PID=0xb00e (Actions Semiconductor)
  - Control Interface: VID=0x1395, PID=0x005d (Solid State System)

### Audio Specifications (Confirmed via macOS)
- **Sample Rate:** 48,000 Hz (48kHz)
- **Bit Depth:** 16-bit PCM
- **Input:** 1 channel (Mono microphone)
- **Output:** 2 channels (Stereo playback)
- **USB Speed:** 480 Mb/s (USB 2.0 High Speed)

### Device Class
- **bDeviceClass:** 0 (Defined at interface level)
- **Serial Number:** ACTIONS-BOS-002
- **Exclusive Access:** Can be claimed by browser (Chrome) for WebUSB

## Current Findings

### 1. WebUSB Compatibility ✅
**CRITICAL DISCOVERY:** Chrome has exclusive USB access!

```
"UsbExclusiveOwner" = "pid 1643, Google Chrome"
```

**Implications:**
- HiNotes webapp uses **WebUSB API** for device communication
- We can inspect WebUSB API calls in Chrome DevTools
- Protocol is accessible via JavaScript (easier reverse engineering!)

### 2. Audio Interface Discovery ✅
The device appears in macOS as:
- Default Input Device (microphone)
- Default Output Device (speakers)
- Manufacturer: Solid State System Co.,Ltd.

This suggests **standard USB Audio Class (UAC)** compliance for audio I/O.

### 3. WebUSB Monitoring Tools Created ✅
**Commit:** 6ce85b6 (2026-08-18)

Created comprehensive WebUSB monitoring toolkit:

#### `/scripts/webusb_monitor.js` - Complete USB Interceptor
- Hooks ALL WebUSB API calls (requestDevice, open, close, claimInterface)
- Captures controlTransferIn/Out with setup packets and data
- Captures transferIn/Out with endpoint numbers and payloads
- Color-coded console output for easy debugging
- Automatic logging to `window.hidocProtocolLog` array
- Export helper: `exportHidocLog()` copies JSON to clipboard
- Auto-save to localStorage on tab blur/close
- Hex dump formatting for binary data analysis

#### Supporting Documentation
- `USB_CAPTURE_INSTRUCTIONS.md` - Detailed step-by-step guide
- `USB_CAPTURE_SESSION.md` - Complete session walkthrough
- `WEBUSB_CAPTURE_STEPS.md` - Quick reference

**Tools Status:** ✅ Ready for use - awaiting manual capture session

### 4. WebUSB Capture Results
**Status:** ⏳ Pending - Manual operation required

**Required Actions:**
1. ✅ WebUSB monitor script created (`scripts/webusb_monitor.js`)
2. ✅ Capture instructions documented
3. ⏳ Actual capture session with HiDoc P1 device (requires physical device access)
4. ⏳ Protocol data analysis
5. ⏳ Command structure documentation
6. ⏳ Rust implementation

**Blockers:** None - toolkit ready, awaiting user with physical HiDoc P1 device access

## Reverse Engineering Strategy

### Phase 1: WebUSB Analysis (EASIEST - START HERE)

**HiNotes webapp uses WebUSB!** This is much easier than USB packet capture.

#### Steps:
1. Open HiNotes webapp in Chrome: https://hinotes.hidock.com
2. Open Chrome DevTools (Cmd+Option+I)
3. Go to Console tab
4. Connect HiDoc P1 device
5. Monitor WebUSB API calls:

```javascript
// Chrome DevTools Console - Monitor USB API
const originalRequestDevice = navigator.usb.requestDevice;
navigator.usb.requestDevice = function(...args) {
  console.log('USB requestDevice called with:', args);
  return originalRequestDevice.apply(this, arguments);
};

// Monitor transfer functions
navigator.usb.getDevices().then(devices => {
  devices.forEach(device => {
    console.log('USB Device:', device);
    console.log('  VendorId:', '0x' + device.vendorId.toString(16));
    console.log('  ProductId:', '0x' + device.productId.toString(16));
    console.log('  Manufacturer:', device.manufacturerName);
    console.log('  Product:', device.productName);
    console.log('  Serial:', device.serialNumber);
  });
});
```

#### Intercept USB Transfers:
```javascript
// Hook into USB transfers
const originalControlTransferIn = USBDevice.prototype.controlTransferIn;
USBDevice.prototype.controlTransferIn = function(setup, length) {
  console.log('controlTransferIn:', setup, 'length:', length);
  return originalControlTransferIn.call(this, setup, length).then(result => {
    console.log('  -> Response:', result);
    return result;
  });
};

const originalControlTransferOut = USBDevice.prototype.controlTransferOut;
USBDevice.prototype.controlTransferOut = function(setup, data) {
  console.log('controlTransferOut:', setup);
  console.log('  Data:', new Uint8Array(data));
  return originalControlTransferOut.call(this, setup, data);
};

const originalTransferIn = USBDevice.prototype.transferIn;
USBDevice.prototype.transferIn = function(endpointNumber, length) {
  console.log('transferIn: endpoint', endpointNumber, 'length', length);
  return originalTransferIn.call(this, endpointNumber, length).then(result => {
    console.log('  -> Received:', new Uint8Array(result.data.buffer));
    return result;
  });
};

const originalTransferOut = USBDevice.prototype.transferOut;
USBDevice.prototype.transferOut = function(endpointNumber, data) {
  console.log('transferOut: endpoint', endpointNumber);
  console.log('  Data:', new Uint8Array(data));
  return originalTransferOut.call(this, endpointNumber, data);
};
```

#### Capture Operations:
With hooks active, perform these operations in webapp:
1. **Connect Device** - Observe USB enumeration
2. **List Files** - Observe file listing commands
3. **Start Recording** - Observe recording start command
4. **Stop Recording** - Observe recording stop command
5. **Transfer File** - Observe file transfer protocol
6. **Delete File** - Observe delete command

#### Export Captured Data:
```javascript
// Save captured protocol to file
let usbLog = [];
// ... (log entries from above hooks)
const blob = new Blob([JSON.stringify(usbLog, null, 2)], {type: 'application/json'});
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'hidoc_p1_protocol.json';
a.click();
```

### Phase 2: USB Packet Capture (IF NEEDED)

If WebUSB analysis isn't sufficient:

#### macOS: Wireshark with USB Capture
```bash
# Install Wireshark (already installed)
brew install wireshark

# Start capture
sudo /Users/sarman/Documents/GitHub/hinotes-desktop-new/scripts/capture_usb_hidoc.sh
```

#### Filter Expression:
```
usb.idVendor == 0x10d6 || usb.idVendor == 0x1395
```

### Phase 3: JavaScript Source Analysis

HiNotes webapp JavaScript bundles contain USB protocol implementation:

#### Download JS Bundles:
```bash
cd /Users/sarman/Documents/GitHub/hidoc
# Already captured in previous analysis
grep -r "transferOut\|transferIn\|controlTransfer" *.js
```

Look for:
- Command constants
- Packet structures
- Endpoint numbers
- Protocol state machines

## Expected Protocol Structure

### Control Interface (0x1395:0x005d)
Likely handles:
- File listing
- File metadata queries
- File transfers
- Delete operations
- Device configuration

### Audio Interface (0x10d6:0xb00e)
Uses **USB Audio Class (UAC)**:
- Standard UAC 1.0 or 2.0 descriptors
- Isochronous endpoints for audio streaming
- Standard audio controls (volume, mute, etc.)

### Typical Command Packet:
```rust
// Expected structure (to be confirmed)
struct CommandPacket {
    magic: [u8; 4],      // Magic bytes e.g. "HDOC" or 0x48444F43
    command_id: u8,      // Command type
    sequence: u16,       // Packet sequence number
    length: u16,         // Payload length
    payload: Vec<u8>,    // Command-specific data
    checksum: u16,       // CRC16 or similar
}
```

### Typical Response Packet:
```rust
struct ResponsePacket {
    magic: [u8; 4],
    status: u8,          // 0x00 = success, 0xFF = error, etc.
    sequence: u16,       // Matching request sequence
    length: u16,
    payload: Vec<u8>,
    checksum: u16,
}
```

## Implementation Plan

### Step 1: WebUSB Protocol Capture (CURRENT - Awaiting Manual Execution)
- [x] ~~Open HiNotes webapp with DevTools~~ (Instructions ready)
- [x] ~~Install WebUSB monitoring hooks~~ (Script ready: `scripts/webusb_monitor.js`)
- [ ] **Perform all device operations** ⏳ REQUIRES PHYSICAL DEVICE
  - [ ] Connect HiDoc P1 via USB
  - [ ] Device enumeration/initialization
  - [ ] List files on device
  - [ ] Start/stop recording
  - [ ] Play audio
  - [ ] Transfer file from device
  - [ ] Delete file from device
  - [ ] Change device settings (if available)
- [ ] Export protocol log (via `exportHidocLog()`)
- [ ] Save to `usb_captures/hidoc_webusb_YYYYMMDD_HHMMSS.json`
- [ ] Analyze command/response patterns

**Capture Guide:** See `WEBUSB_CAPTURE_STEPS.md` for quick start or `USB_CAPTURE_SESSION.md` for detailed walkthrough

### Step 2: Protocol Documentation (BLOCKED - Awaiting Capture Data)
- [ ] Document all discovered commands
- [ ] Map command IDs to operations
- [ ] Identify control vs. bulk vs. interrupt endpoints
- [ ] Document packet structures (headers, magic bytes, checksums)
- [ ] Map USB setup packet fields to operations
- [ ] Document error codes and status values
- [ ] Create protocol specification document

### Step 3: Rust Implementation (BLOCKED - Awaiting Protocol Docs)
- [ ] Update `src-tauri/src/usb/protocol.rs` with actual structures
- [ ] Implement command builders (based on captured packets)
- [ ] Implement response parsers
- [ ] Add protocol tests with captured data
- [ ] Implement checksum/CRC validation (if present)

### Step 4: Device Communication (BLOCKED - Awaiting Rust Implementation)
- [ ] Implement `rusb` device handle acquisition
- [ ] Claim USB control interface (VID=0x1395, PID=0x005d)
- [ ] Send test commands
- [ ] Verify responses match webapp behavior
- [ ] Implement retry logic and error handling

### Step 5: Integration (BLOCKED - Awaiting Device Communication)
- [ ] Integrate with Tauri commands
- [ ] Add UI for device operations
- [ ] Test full workflow: list files → transfer → delete
- [ ] Handle device hotplug events
- [ ] Add device status monitoring

## Tools and Scripts

### Created Tools (Ready for Use) ✅

#### 1. WebUSB Monitor Script
**File:** `scripts/webusb_monitor.js`  
**Purpose:** Comprehensive WebUSB API interceptor for Chrome DevTools

**Features:**
- Hooks all USB operations: `requestDevice`, `open`, `close`, `claimInterface`, `selectConfiguration`
- Captures control transfers: `controlTransferIn`, `controlTransferOut`
- Captures bulk/interrupt transfers: `transferIn`, `transferOut`
- Color-coded console output (blue=request, green=response, red=error)
- Automatic logging to `window.hidocProtocolLog` array with timestamps
- Hex dump formatting for binary data
- Export helper: `exportHidocLog()` copies to clipboard
- Auto-save to localStorage on tab blur
- Restores previous session on page reload

**Usage:**
```bash
# Copy to clipboard
cat scripts/webusb_monitor.js | pbcopy

# Then paste in Chrome DevTools Console
# See WEBUSB_CAPTURE_STEPS.md for details
```

#### 2. Capture Documentation
**Files:**
- `WEBUSB_CAPTURE_STEPS.md` - Quick 5-minute guide
- `USB_CAPTURE_SESSION.md` - Detailed walkthrough with explanations
- `USB_CAPTURE_INSTRUCTIONS.md` - Step-by-step procedures

**Contents:**
- Prerequisites and setup
- Browser configuration (WebUSB flags)
- Monitor installation steps
- Operations to capture (connect, record, play, transfer, delete)
- Export and analysis procedures
- Troubleshooting common issues

#### 3. Capture Storage
**Directory:** `usb_captures/`  
**Format:** JSON files with naming: `hidoc_webusb_YYYYMMDD_HHMMSS.json`

**Structure:**
```json
[
  {
    "index": 0,
    "timestamp": 1692345678901,
    "time": "2026-08-18T15:30:00.000Z",
    "type": "requestDevice",
    "args": [...]
  },
  {
    "index": 1,
    "type": "controlTransferOut",
    "setup": {...},
    "data": [0x01, 0x02, 0x03],
    "hex": "01 02 03"
  }
]
```

### Legacy Tools (Optional)

#### Device Enumeration
```bash
./scripts/enumerate_hidoc.sh
```

#### USB Packet Capture (Wireshark - if WebUSB insufficient)
```bash
./scripts/capture_usb_hidoc.sh
```

## Resources

- [WebUSB API Specification](https://wicg.github.io/webusb/)
- [USB Made Simple](http://www.usbmadesimple.co.uk/)
- [USB Audio Class 1.0 Spec](https://www.usb.org/document-library/audio-device-class-spec-10)
- [rusb Documentation](https://docs.rs/rusb/)
- Chrome DevTools → Application → USB (to see connected devices)

## Progress Tracking

- [x] Device VID/PID identified (Audio: 0x10d6:0xb00e, Control: 0x1395:0x005d)
- [x] Audio specifications confirmed (48kHz, 16-bit, mono in, stereo out)
- [x] WebUSB usage discovered (Chrome exclusive access)
- [x] WebUSB monitoring toolkit created (scripts/webusb_monitor.js)
- [x] Capture instructions documented (3 guides created)
- [ ] **WebUSB protocol captured** ⏳ AWAITING MANUAL SESSION WITH PHYSICAL DEVICE
- [ ] Command structures documented (blocked by capture)
- [ ] Rust implementation created (blocked by protocol docs)
- [ ] End-to-end testing completed (blocked by implementation)

## Notes

### Important Observations:
1. **WebUSB is the key!** Much easier than raw USB capture.
2. **Standard UAC for audio** - Use existing audio APIs, not custom protocol.
3. **Chrome has exclusive access** - Webapp must release device before Rust app can claim it.
4. **Dual interfaces** - Separate audio (UAC) from control (custom protocol).
5. **Monitoring toolkit complete** - All infrastructure ready for capture session.

### Next Immediate Steps:

#### For User with Physical HiDoc P1 Device Access:
1. **Read Quick Start Guide:** Open `WEBUSB_CAPTURE_STEPS.md` for 5-minute quickstart
2. **Or Follow Detailed Guide:** Open `USB_CAPTURE_SESSION.md` for complete walkthrough
3. **Execute Capture:**
   ```bash
   # Copy monitor script to clipboard
   cat scripts/webusb_monitor.js | pbcopy
   
   # Open HiNotes in Chrome
   open -a "Google Chrome" https://hinotes.hidock.com
   
   # Then:
   # - Open DevTools (Cmd+Option+I)
   # - Paste script in Console
   # - Refresh page
   # - Perform device operations
   # - Run: exportHidocLog()
   # - Save: pbpaste > usb_captures/hidoc_webusb_$(date +%Y%m%d_%H%M%S).json
   ```

#### After Capture Data Available:
1. **Analyze Captured Protocol** (automated analysis script TBD)
2. **Document Command Structures** (create USB_PROTOCOL_SPEC.md)
3. **Implement in Rust** (update src-tauri/src/usb/protocol.rs)
4. **Test with Real Device** (verify Rust implementation matches webapp)

### Estimated Effort Remaining:
- **Manual capture session:** 30-60 minutes (requires physical device)
- **Protocol analysis:** 2-4 hours (after capture data available)
- **Rust implementation:** 4-8 hours (after protocol documented)
- **Testing & integration:** 4-6 hours (after implementation)

**Total:** ~10-18 hours of development after initial capture session

### Current Blocker:
**Physical HiDoc P1 device access required for capture session.** All infrastructure and tooling ready.

---

**Last Updated:** 2026-08-18 (15:30 CDT)  
**Monitoring Tools:** Commit 6ce85b6 (2026-08-18)  
**Contributors:** Claude Sonnet 4.5 + Shaun Arman
