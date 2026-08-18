# HiDoc P1 USB Protocol Analysis

**Status:** In Progress  
**Last Updated:** 2026-08-18

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

### 1. WebUSB Compatibility
**CRITICAL DISCOVERY:** Chrome has exclusive USB access!

```
"UsbExclusiveOwner" = "pid 1643, Google Chrome"
```

**Implications:**
- HiNotes webapp uses **WebUSB API** for device communication
- We can inspect WebUSB API calls in Chrome DevTools
- Protocol is accessible via JavaScript (easier reverse engineering!)

### 2. Audio Interface Discovery
The device appears in macOS as:
- Default Input Device (microphone)
- Default Output Device (speakers)
- Manufacturer: Solid State System Co.,Ltd.

This suggests **standard USB Audio Class (UAC)** compliance for audio I/O.

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

### Step 1: WebUSB Protocol Capture (CURRENT)
- [ ] Open HiNotes webapp with DevTools
- [ ] Install WebUSB monitoring hooks
- [ ] Perform all device operations
- [ ] Export protocol log
- [ ] Analyze command/response patterns

### Step 2: Protocol Documentation
- [ ] Document all discovered commands
- [ ] Map command IDs to operations
- [ ] Document packet structures
- [ ] Create protocol specification document

### Step 3: Rust Implementation
- [ ] Update `src-tauri/src/usb/protocol.rs` with actual structures
- [ ] Implement command builders
- [ ] Implement response parsers
- [ ] Add protocol tests with captured data

### Step 4: Device Communication
- [ ] Implement `rusb` device handle acquisition
- [ ] Claim USB interface
- [ ] Send test commands
- [ ] Verify responses match webapp behavior

### Step 5: Integration
- [ ] Integrate with Tauri commands
- [ ] Add UI for device operations
- [ ] Test full workflow: list files → transfer → delete

## Tools and Scripts

### Device Enumeration
```bash
./scripts/enumerate_hidoc.sh
```

### USB Packet Capture
```bash
./scripts/capture_usb_hidoc.sh
```

### WebUSB Monitoring
1. Open: https://hinotes.hidock.com
2. DevTools → Console
3. Paste monitoring hooks (see Phase 1)
4. Perform operations
5. Export logs

## Resources

- [WebUSB API Specification](https://wicg.github.io/webusb/)
- [USB Made Simple](http://www.usbmadesimple.co.uk/)
- [USB Audio Class 1.0 Spec](https://www.usb.org/document-library/audio-device-class-spec-10)
- [rusb Documentation](https://docs.rs/rusb/)
- Chrome DevTools → Application → USB (to see connected devices)

## Progress Tracking

- [x] Device VID/PID identified
- [x] Audio specifications confirmed (48kHz, 16-bit, mono in, stereo out)
- [x] WebUSB usage discovered (Chrome exclusive access)
- [ ] WebUSB protocol captured
- [ ] Command structures documented
- [ ] Rust implementation created
- [ ] End-to-end testing completed

## Notes

### Important Observations:
1. **WebUSB is the key!** Much easier than raw USB capture.
2. **Standard UAC for audio** - Use existing audio APIs, not custom protocol.
3. **Chrome has exclusive access** - Webapp must release device before Rust app can claim it.
4. **Dual interfaces** - Separate audio (UAC) from control (custom protocol).

### Next Immediate Step:
**Open HiNotes webapp in Chrome DevTools and capture WebUSB calls!**

This will give us the complete protocol in minutes, compared to hours/days of USB packet analysis.

---

**Last Updated:** 2026-08-18  
**Contributor:** Claude Sonnet 4.5 + Shaun Arman
