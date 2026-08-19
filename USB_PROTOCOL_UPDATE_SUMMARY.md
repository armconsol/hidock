# USB Protocol Structure Update Summary

## Date: 2026-08-18

## Task: Update Protocol Structures with Device Reality Warning

Updated HiDoc P1 USB protocol implementation to clearly distinguish between confirmed device identifiers and hypothetical protocol structures.

## Changes Made

### 1. Protocol Structures (`src-tauri/src/usb/protocol.rs`)

#### CommandPacket Structure
- **Added critical warnings** about hypothetical nature
- Documented that structure is PLACEHOLDER until confirmed via:
  - USB packet capture (Wireshark/USBPcap)
  - Real device testing
  - HiNotes app reverse engineering
- Clarified that header (0xAA55), command bytes, and checksum algorithm are all UNKNOWN
- Added confirmed device information in documentation:
  - Audio Interface: VID 0x10d6, PID 0xb00e
  - Control Interface: VID 0x1395, PID 0x005d
  - Serial: ACTIONS-BOS-002
  - Manufacturer: HiDock (not "HiDoc")

#### ResponsePacket Structure  
- **Added critical warning** about hypothetical format
- Documented assumptions:
  - First byte = status code (0 = success) - UNCONFIRMED
  - Remaining bytes = response payload - UNCONFIRMED
- Emphasized need for USB packet capture to determine real format

### 2. Device Detection (`src-tauri/src/usb/detector.rs`)

#### Dual Interface Support
- Updated to detect **both** HiDoc P1 USB interfaces:
  - Audio Interface (VID 0x10d6, PID 0xb00e)
  - Control/Storage Interface (VID 0x1395, PID 0x005d)

#### New Methods
- `find_device()` - Returns control interface (preferred)
- `find_audio_device()` - Returns audio interface specifically
- `find_all_interfaces()` - Returns tuple of (audio_device, control_device)

#### Enhanced Validation
- Checks manufacturer name for "HiDock" (note spelling)
- Validates against both interface VID/PID combinations
- Logs warnings for unexpected manufacturer/product strings

### 3. Device Information (`src-tauri/src/usb/mod.rs`)

#### New Interface Detection Methods
- `is_hidoc_p1()` - Updated to check for EITHER interface
- `is_audio_interface()` - Check specifically for audio interface (0x10d6:0xb00e)
- `is_control_interface()` - Check specifically for control interface (0x1395:0x005d)

#### Enhanced Validation
- `validate()` now accepts manufacturer strings from either chip:
  - "HiDock" (preferred)
  - "Actions Semiconductor" (audio chip)
  - "Solid State System" (control chip)
- Accepts product name variations: "HiDock", "HiDoc", "P1"
- Improved warning messages with VID/PID in hex format

### 4. Constants Updated

All files now use explicit constants:
- `HIDOC_P1_AUDIO_VID` = 0x10d6
- `HIDOC_P1_AUDIO_PID` = 0xb00e  
- `HIDOC_P1_CONTROL_VID` = 0x1395
- `HIDOC_P1_CONTROL_PID` = 0x005d
- `HIDOC_MANUFACTURER` = "HiDock"
- `HIDOC_PRODUCT` = "P1"

Backward compatibility: Old `HIDOC_P1_VID`/`HIDOC_P1_PID` deprecated but still point to control interface.

## What Was NOT Changed

### Protocol Implementation
- Command byte values (0x01-0x20) remain PLACEHOLDERS
- Packet structure (header, length, checksum) remains HYPOTHETICAL
- Endpoint addresses (0x01, 0x81, 0x82) are UNCONFIRMED

### Next Steps for Protocol Discovery
1. **USB Packet Capture**: Use Wireshark/USBPcap to capture HiNotes app communication
2. **Endpoint Discovery**: Analyze USB device descriptors (lsusb -v)
3. **Protocol Reverse Engineering**: Test actual commands with real device
4. **Verification**: Confirm packet format, checksums, and response structures

## Files Modified

- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/usb/protocol.rs` (+90% documentation)
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/usb/detector.rs` (+122 lines)
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/usb/mod.rs` (+32 lines)
- `/Users/sarman/Documents/GitHub/hidoc/src-tauri/src/usb/mass_storage.rs` (minor reformatting)

## Testing Status

- ✅ Code compiles without errors
- ✅ No USB-specific compilation errors
- ⚠️ Unit tests pass (with placeholder implementations)
- ❌ **NOT tested with real device** (hardware required)

## Safety Warnings Added

All hypothetical protocol structures now include:
- ⚠️ CRITICAL WARNING headers
- Explanation of what's placeholder vs confirmed
- Instructions for discovering real protocol
- DO NOT USE WITH REAL HARDWARE warnings

## Device Information (Confirmed)

**Audio Interface** (VID 0x10d6, PID 0xb00e)
- Manufacturer: Actions Semiconductor Co., Ltd.
- Product: HiDock_P1
- Serial: ACTIONS-BOS-002
- Speed: USB 2.0 High Speed (480 Mb/s)
- Purpose: Audio capture (mono, 48kHz, 16-bit) / playback (stereo)

**Control Interface** (VID 0x1395, PID 0x005d)
- Manufacturer: Solid State System Co.,Ltd.
- Product: HiDock P1
- Speed: USB 1.1 Full Speed (12 Mb/s)  
- Purpose: Device control, configuration, file storage

Source: macOS `system_profiler SPUSBDataType`

## Verification Needed

Before using protocol implementation with real device:
1. Capture USB packets during HiNotes operation
2. Identify actual command/response format
3. Determine real endpoint addresses
4. Verify checksum algorithm
5. Test with device in safe, reversible operations

## Impact

- **Code Safety**: ✅ Improved (clear warnings about unknowns)
- **Device Detection**: ✅ Improved (supports both interfaces)
- **Protocol Accuracy**: ⚠️ Unchanged (still hypothetical, now documented as such)
- **Future Work**: ✅ Clear path forward (packet capture required)
