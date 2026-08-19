# USB Identifier Update Summary

## Overview

Updated the HiDoc P1 USB device detection code with actual device identifiers discovered through system USB enumeration.

## Device Identifiers

### Discovered Interfaces

The HiDoc P1 exposes **two separate USB interfaces** with different VID/PID pairs:

1. **Audio Interface** (Actions Semiconductor Co., Ltd.)
   - Vendor ID: `0x10d6`
   - Product ID: `0xb00e`
   - Product Name: `HiDock_P1` or `HiDoc P1`
   - Serial Number: `ACTIONS-BOS-002`
   - USB Speed: 480 Mb/s (USB 2.0 High Speed)
   - Purpose: Audio capture (mono, 48kHz, 16-bit PCM) and playback (stereo, 48kHz, 16-bit PCM)

2. **Control/Storage Interface** (Solid State System Co.,Ltd.)
   - Vendor ID: `0x1395`
   - Product ID: `0x005d`
   - Product Name: `HiDock P1`
   - USB Speed: 12 Mb/s (USB 1.1 Full Speed)
   - Purpose: Device control, configuration, and file storage (mass storage mode)

## Code Changes

### 1. Constants Updated (`src-tauri/src/usb/mod.rs`)

Added new constants for both interfaces:

```rust
/// Audio interface Vendor ID (Actions Semiconductor Co., Ltd.)
pub const HIDOC_P1_AUDIO_VID: u16 = 0x10d6;

/// Audio interface Product ID
pub const HIDOC_P1_AUDIO_PID: u16 = 0xb00e;

/// Control/Storage interface Vendor ID (Solid State System Co.,Ltd.)
pub const HIDOC_P1_CONTROL_VID: u16 = 0x1395;

/// Control/Storage interface Product ID
pub const HIDOC_P1_CONTROL_PID: u16 = 0x005d;
```

Deprecated old constants for backward compatibility:

```rust
#[deprecated(note = "Use HIDOC_P1_CONTROL_VID instead")]
pub const HIDOC_P1_VID: u16 = HIDOC_P1_CONTROL_VID;

#[deprecated(note = "Use HIDOC_P1_CONTROL_PID instead")]
pub const HIDOC_P1_PID: u16 = HIDOC_P1_CONTROL_PID;
```

### 2. New InterfaceType Enum

Added enum to distinguish between the two interfaces:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    /// Audio interface (VID=0x10d6, PID=0xb00e)
    Audio,
    /// Control/Storage interface (VID=0x1395, PID=0x005d)
    Control,
}
```

### 3. DeviceInfo Methods Enhanced

Updated `DeviceInfo` impl with methods to handle both interfaces:

```rust
impl DeviceInfo {
    /// Check if this is a HiDoc P1 device (either interface)
    pub fn is_hidoc_p1(&self) -> bool {
        self.is_audio_interface() || self.is_control_interface()
    }

    /// Check if this is the audio interface
    pub fn is_audio_interface(&self) -> bool {
        self.vendor_id == HIDOC_P1_AUDIO_VID && self.product_id == HIDOC_P1_AUDIO_PID
    }

    /// Check if this is the control/storage interface
    pub fn is_control_interface(&self) -> bool {
        self.vendor_id == HIDOC_P1_CONTROL_VID && self.product_id == HIDOC_P1_CONTROL_PID
    }

    /// Get the interface type for this device
    pub fn interface_type(&self) -> Option<InterfaceType> {
        if self.is_audio_interface() {
            Some(InterfaceType::Audio)
        } else if self.is_control_interface() {
            Some(InterfaceType::Control)
        } else {
            None
        }
    }
}
```

### 4. Detector Module Updated (`src-tauri/src/usb/detector.rs`)

Added methods to find specific interfaces:

```rust
/// Find the HiDoc P1 audio interface
pub fn find_audio_device(&self) -> Result<DeviceInfo, UsbError>

/// Find all HiDoc P1 interfaces (both audio and control)
pub fn find_all_interfaces(&self) -> Result<(DeviceInfo, DeviceInfo), UsbError>
```

Updated imports to use new constants:

```rust
use super::{
    DeviceInfo, DeviceState, InterfaceType, UsbError,
    HIDOC_P1_AUDIO_VID, HIDOC_P1_AUDIO_PID,
    HIDOC_P1_CONTROL_VID, HIDOC_P1_CONTROL_PID,
    HIDOC_MANUFACTURER, USB_TIMEOUT_MS,
};
```

### 5. Mass Storage Detection (`src-tauri/src/usb/mass_storage.rs`)

Volume detection already handles the device correctly:
- Checks for "HIDOC" or "P1" in volume names
- Platform-specific mount point detection for macOS, Linux, and Windows
- The control interface (0x1395:0x005d) presents as mass storage device

### 6. Test Cases Added

Added comprehensive tests for both interfaces:

```rust
#[test]
fn test_device_info_validation_control() { ... }

#[test]
fn test_device_info_validation_audio() { ... }
```

## Implementation Status

### ✅ Completed

- [x] Constants defined for both interfaces
- [x] InterfaceType enum created
- [x] DeviceInfo methods for interface detection
- [x] Detector module methods for finding specific interfaces
- [x] Test cases for both interfaces
- [x] Documentation updated with actual VID/PID values

### 🔄 Remaining TODOs

The following TODOs remain but require actual USB library integration (rusb):

1. **Device Enumeration** (`detector.rs:100-122`)
   - TODO: Implement with rusb to scan for devices
   - Currently returns empty Vec placeholder

2. **Device Information Extraction** (`detector.rs:126-160`)
   - TODO: Implement with rusb to read device descriptors
   - Commented out implementation ready for rusb integration

3. **Mass Storage Class Detection** (`detector.rs:163-184`)
   - TODO: Implement with rusb to check interface class codes
   - Currently returns false placeholder

4. **Protocol Handler** (`protocol.rs:201-227`)
   - TODO: Initialize rusb context and device handle
   - TODO: Claim USB interface for communication

5. **USB Transfers** (`protocol.rs:245-276`, `protocol.rs:283-297`)
   - TODO: Implement actual bulk transfers for commands and audio
   - Currently returns placeholder empty responses

## Testing

### Current State

- ✅ Code compiles with placeholder implementations
- ✅ Type system enforces correct interface handling
- ✅ Unit tests verify VID/PID matching logic
- ⚠️  Cannot test actual USB operations without rusb library

### Next Steps for Testing

1. Add `rusb` dependency to `Cargo.toml`
2. Uncomment TODO implementations in detector.rs and protocol.rs
3. Test with actual HiDoc P1 device connected
4. Verify both interfaces are detected independently
5. Test mass storage fallback mode

## Usage Example

```rust
use hidoc::usb::{DeviceDetector, InterfaceType};

// Create detector
let detector = DeviceDetector::new()?;

// Find both interfaces
let (audio, control) = detector.find_all_interfaces()?;

// Check interface types
assert_eq!(audio.interface_type(), Some(InterfaceType::Audio));
assert_eq!(control.interface_type(), Some(InterfaceType::Control));

// Or find specific interface
let audio_device = detector.find_audio_device()?;
assert!(audio_device.is_audio_interface());
```

## Migration Guide

### For Existing Code

Code using the old deprecated constants will continue to work but should migrate:

```rust
// Old (deprecated)
if device.vendor_id == HIDOC_P1_VID && device.product_id == HIDOC_P1_PID {
    // ...
}

// New (recommended)
if device.is_hidoc_p1() {
    match device.interface_type() {
        Some(InterfaceType::Audio) => {
            // Handle audio interface
        }
        Some(InterfaceType::Control) => {
            // Handle control interface
        }
        None => {
            // Unknown device
        }
    }
}
```

## References

- USB device discovery: `system_profiler SPUSBDataType` (macOS)
- Audio interface specifications: macOS Audio MIDI Setup
- USB Audio Class: USB Class specification for Audio Devices
- Mass storage: USB Mass Storage Class specification
