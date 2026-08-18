# HiDoc P1 USB Device Information

**Discovered:** 2026-08-18  
**Platform:** macOS (Apple Silicon)

## USB Device Identification

The HiDoc P1 exposes **TWO USB interfaces**:

### Interface 1: Audio (Actions Semiconductor)
```
Product Name:     HiDock_P1
Vendor ID (VID):  0x10d6 (Actions Semiconductor Co., Ltd.)
Product ID (PID): 0xb00e
Version:          2.00
Serial Number:    ACTIONS-BOS-002
Speed:            480 Mb/s (USB 2.0 High Speed)
Manufacturer:     HiDock
Location ID:      0x02120000 / 2
Current Draw:     100 mA (500 mA available)
```

**Purpose:** Audio capture/playback functionality

### Interface 2: Control/Storage (Solid State System)
```
Product Name:     HiDock P1
Vendor ID (VID):  0x1395 (Solid State System Co.,Ltd.)
Product ID (PID): 0x005d
Version:          1.00
Speed:            12 Mb/s (USB 1.1 Full Speed)
Manufacturer:     Solid State System Co.,Ltd.
Location ID:      0x02110000 / 3
Current Draw:     100 mA (500 mA available)
```

**Purpose:** Device control, configuration, and possibly file storage

## Code Updates

Updated constants in `src-tauri/src/usb/mod.rs`:
```rust
pub const HIDOC_P1_AUDIO_VID: u16 = 0x10d6;
pub const HIDOC_P1_AUDIO_PID: u16 = 0xb00e;
pub const HIDOC_P1_CONTROL_VID: u16 = 0x1395;
pub const HIDOC_P1_CONTROL_PID: u16 = 0x005d;
pub const HIDOC_MANUFACTURER: &str = "HiDock"; // Note: "HiDock" not "HiDoc"
```

## Mass Storage Status

**NOT mounted as mass storage device** - No volume appears in `/Volumes/` or `diskutil list`.

This indicates:
- Device uses custom USB protocol for file transfer
- Requires reverse engineering USB communication
- Mass storage fallback NOT available for this device

## Next Steps for Protocol Reverse Engineering

### Option 1: Wireshark with USBPcap (Recommended for macOS)

**Install Wireshark:**
```bash
brew install wireshark
```

**Capture USB traffic:**
1. Launch Wireshark with sudo: `sudo wireshark`
2. Select USB capture interface
3. Apply filter: `usb.idVendor == 0x10d6 || usb.idVendor == 0x1395`
4. Perform operations on HiDoc P1 (record, play, transfer files)
5. Save capture as `.pcapng` file

### Option 2: macOS PacketLogger (Apple Developer Tools)

**Install:**
1. Download "Additional Tools for Xcode" from Apple Developer
2. Extract and locate `PacketLogger.app`
3. Launch PacketLogger

**Capture:**
1. Click "Start" in PacketLogger
2. Perform HiDoc P1 operations
3. Save capture log
4. Export as text for analysis

### Option 3: libusb Direct Enumeration

**Using rusb in Rust:**
```bash
cd src-tauri
cargo add rusb
cargo test test_enumerate_hidoc_p1 -- --nocapture
```

This will list device endpoints, interfaces, and configurations.

## Operations to Capture

When packet capturing is active, perform these operations:

1. **Device Detection:**
   - Plug in device
   - Capture USB enumeration sequence

2. **Recording:**
   - Press record button on device
   - Capture command packets

3. **Playback:**
   - Play recorded audio
   - Capture audio streaming packets

4. **File Transfer:**
   - Initiate file sync/transfer via HiNotes app (if available)
   - Capture file list queries and data transfers

5. **Configuration:**
   - Change device settings (if any UI available)
   - Capture configuration packets

## Expected Protocol Elements

Based on dual-interface design:

**Audio Interface (0x10d6:0xb00e):**
- Likely uses USB Audio Class (UAC) or custom isochronous transfers
- Expect audio data in 16-bit PCM format
- Sample rate: 16kHz or 44.1kHz (to be determined)
- May use standard UAC descriptors

**Control Interface (0x1395:0x005d):**
- Custom bulk or control transfers
- Command/response protocol
- File listing and transfer commands
- Device status queries
- Configuration writes

## Analysis Tools

Once packet capture is obtained:

1. **Wireshark Analysis:**
   - USB protocol dissector
   - Filter by VID/PID
   - Look for repeating patterns in bulk/control transfers

2. **Pattern Recognition:**
   - Command headers (magic bytes)
   - Length fields
   - Checksums/CRCs
   - Response codes

3. **Documentation:**
   - Map commands to operations
   - Document packet structures
   - Create protocol specification

## rusb Integration

Once protocol is understood, implement in `src-tauri/src/usb/protocol.rs`:

```rust
// Example command structure (to be determined)
pub struct CommandPacket {
    pub magic: [u8; 4],        // e.g., 0x48 0x44 0x4F 0x43 ("HDOC")
    pub command: u8,           // Command ID
    pub length: u16,           // Payload length
    pub payload: Vec<u8>,      // Command-specific data
    pub checksum: u16,         // CRC or simple checksum
}
```

## Current Status

- ✅ Device VID/PIDs identified
- ✅ Code constants updated
- ⏳ Packet capture pending (need Wireshark installed)
- ⏳ Protocol analysis pending
- ⏳ rusb implementation pending

## References

- USB_INTEGRATION.md - Comprehensive reverse engineering guide
- src-tauri/src/usb/ - USB module implementation
- [USB Made Simple](http://www.usbmadesimple.co.uk/) - USB protocol primer
- [rusb documentation](https://docs.rs/rusb/) - Rust USB library

---

**Last Updated:** 2026-08-18  
**Device Owner:** Shaun Arman  
**Project:** HiNotes Desktop
