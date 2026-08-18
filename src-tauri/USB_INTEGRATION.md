# HiDoc P1 USB Device Integration

## Overview

The HiDoc P1 is a USB audio transcription device that requires protocol reverse engineering to enable direct integration with the HiNotes Desktop application. This document outlines the approach for understanding and implementing the USB protocol.

## Device Information

- **Device**: HiDoc P1 USB Audio Transcription Device
- **Connection**: USB 2.0/3.0
- **Expected Functionality**: Audio recording with potential control commands
- **VID/PID**: To be determined through device enumeration

## Reverse Engineering Approach

### Phase 1: Device Discovery

1. **Device Enumeration**
   - Connect device and identify VID/PID using system tools
   - macOS: `system_profiler SPUSBDataType`
   - Linux: `lsusb -v`
   - Windows: Device Manager or USBView
   
2. **Interface Analysis**
   - Determine device class (Audio, HID, Vendor-specific)
   - Identify endpoints (IN/OUT, bulk/interrupt/isochronous)
   - Document configuration descriptors

3. **Driver Behavior Study**
   - Observe how official software/drivers interact with device
   - Note any required initialization sequences

### Phase 2: Protocol Capture

#### Required Tools

##### macOS
- **PacketLogger** (Xcode Additional Tools)
  - Location: `/Applications/Xcode.app/Contents/Applications/PacketLogger.app`
  - Alternative: Download from Apple Developer
  - Captures USB, Bluetooth packets at system level
  
- **Wireshark with USBPcap**
  - Install Wireshark: `brew install wireshark`
  - Note: macOS USB capture requires kernel extensions or PacketLogger

##### Linux
- **Wireshark with usbmon**
  ```bash
  sudo modprobe usbmon
  sudo wireshark
  # Select usbmon interface
  ```
  
- **tcpdump/dumpcap**
  ```bash
  sudo tcpdump -i usbmon1 -w capture.pcap
  ```

##### Windows
- **USBPcap** (https://desowin.org/usbpcap/)
  - Kernel-mode USB packet capture
  - Wireshark integration
  
- **Wireshark**
  - Install with USBPcap option enabled
  - Select USBPcap interface for target bus

#### Capture Strategy

1. **Baseline Capture**
   - Capture device connection/enumeration
   - Document initialization sequence
   - Identify any control transfers

2. **Operational Capture**
   - Start recording session
   - Capture audio data transfers
   - Stop recording session
   - Document command sequences

3. **Feature Testing**
   - Test all device features (play, pause, delete, etc.)
   - Capture each operation separately
   - Correlate UI actions with USB packets

### Phase 3: Packet Analysis

#### Analysis Workflow

1. **Filter Relevant Traffic**
   ```
   Wireshark filter: usb.device_address == X && usb.bus_id == Y
   ```

2. **Identify Packet Types**
   - **Control Transfers**: Device configuration, commands
   - **Bulk Transfers**: Audio data, file transfers
   - **Interrupt Transfers**: Status updates, notifications
   - **Isochronous Transfers**: Real-time audio streaming

3. **Pattern Recognition**
   - Repeated sequences = protocol handshakes
   - Variable data = payload/content
   - Fixed headers = packet structure

4. **Data Structure Extraction**
   - Document packet formats
   - Identify magic bytes/signatures
   - Determine field meanings through correlation

#### Expected Protocol Elements

##### Device Identification
```
VID: 0x???? (To be determined)
PID: 0x???? (To be determined)
Class: Vendor-specific / Audio / Mass Storage
```

##### Command Structure (Hypothetical)
```
[Header: 2 bytes] [Command: 1 byte] [Length: 2 bytes] [Payload: N bytes] [Checksum: 1 byte]
```

##### Audio Data Format
```
- Sample Rate: 16kHz/44.1kHz/48kHz (to be determined)
- Bit Depth: 16-bit PCM (typical)
- Channels: Mono/Stereo
- Encoding: Raw PCM / Compressed (ADPCM, etc.)
```

### Phase 4: Protocol Implementation

#### libusb Integration via rusb

The `rusb` crate provides Rust bindings to libusb 1.0.

##### Key Components

1. **Device Detection**
   ```rust
   // Find device by VID/PID
   let devices = rusb::devices()?;
   for device in devices.iter() {
       let desc = device.device_descriptor()?;
       if desc.vendor_id() == HIDOC_VID && desc.product_id() == HIDOC_PID {
           // Found HiDoc P1
       }
   }
   ```

2. **Device Handle Acquisition**
   ```rust
   let handle = device.open()?;
   handle.claim_interface(interface_num)?;
   ```

3. **Data Transfer**
   - **Control**: Configuration, commands
   - **Bulk**: File/audio data transfer
   - **Interrupt**: Event notifications
   - **Isochronous**: Streaming audio

4. **Error Handling**
   - Device not found
   - Permission denied (requires udev rules on Linux)
   - Transfer timeouts
   - Protocol errors

## Fallback Approach: USB Mass Storage

If direct USB protocol proves too complex or proprietary, fall back to mass storage interface.

### Mass Storage Detection

Many recording devices expose a USB Mass Storage Class interface for file access.

#### Implementation Strategy

1. **Monitor for Mass Storage Device**
   ```rust
   // Check if device exposes bInterfaceClass == 0x08 (Mass Storage)
   let config = device.active_config_descriptor()?;
   for interface in config.interfaces() {
       for desc in interface.descriptors() {
           if desc.class_code() == 0x08 {
               // Mass Storage detected
           }
       }
   }
   ```

2. **Mount Point Detection**
   - macOS: `/Volumes/HIDOC_P1` (or similar)
   - Linux: `/media/${USER}/HIDOC_P1` or `/mnt/`
   - Windows: `E:\` (or next available drive letter)

3. **File System Monitoring**
   ```rust
   // Use notify crate for file system events
   // Watch for .wav/.mp3 files in mount point
   ```

4. **Audio File Import**
   - Monitor for new files
   - Copy to local storage
   - Extract metadata
   - Import into HiNotes

### Advantages
- No protocol reverse engineering required
- Standard filesystem APIs
- Cross-platform compatibility
- Simpler implementation

### Disadvantages
- No real-time streaming
- Limited device control
- Depends on device exposing mass storage
- Slower workflow (record, save, import)

## Dependencies

### Rust Crates Required

```toml
[dependencies]
# USB Communication
rusb = "0.9"

# File system monitoring (for mass storage fallback)
notify = "6.0"

# Audio processing
hound = "3.5"        # WAV file I/O
symphonia = "0.5"    # Multiple audio format support

# Async runtime (already in project)
tokio = { version = "1.36", features = ["full"] }

# Error handling (already in project)
anyhow = "1.0"
thiserror = "1.0"

# Serialization (already in project)
serde = { version = "1", features = ["derive"] }

# Logging (already in project)
log = "0.4"
```

### System Dependencies

#### macOS
```bash
# libusb via Homebrew
brew install libusb

# Xcode for PacketLogger
xcode-select --install
```

#### Linux
```bash
# Debian/Ubuntu
sudo apt-get install libusb-1.0-0-dev

# Fedora/RHEL
sudo dnf install libusb1-devel

# udev rules required for non-root access
sudo nano /etc/udev/rules.d/99-hidoc-p1.rules
# SUBSYSTEM=="usb", ATTR{idVendor}=="????", ATTR{idProduct}=="????", MODE="0666"
sudo udevadm control --reload-rules
```

#### Windows
```powershell
# libusb via vcpkg
vcpkg install libusb:x64-windows

# Or use pre-built binaries from libusb.info
# Place libusb-1.0.dll in system PATH or application directory
```

## Testing Strategy (Without Physical Device)

### Mock Device Simulation

1. **USB Device Emulation**
   - Linux: Use `dummy_hcd` kernel module
   - Create virtual USB device with target VID/PID
   - Respond to control transfers with canned responses

2. **Protocol Simulator**
   ```rust
   #[cfg(test)]
   mod mock_usb {
       struct MockHiDocP1 {
           state: DeviceState,
           audio_buffer: Vec<u8>,
       }
       
       impl MockHiDocP1 {
           fn handle_control(&mut self, request: u8) -> Result<Vec<u8>> {
               // Simulate device responses
           }
           
           fn handle_bulk_read(&mut self) -> Result<Vec<u8>> {
               // Return mock audio data
           }
       }
   }
   ```

3. **Recorded Packet Playback**
   - Capture real device packets once available
   - Store as test fixtures
   - Replay for regression testing

### Integration Testing

1. **Device Detection Tests**
   ```rust
   #[test]
   fn test_device_enumeration() {
       let detector = HiDocDetector::new();
       // Mock USB device list
       assert!(detector.find_device().is_ok());
   }
   ```

2. **Protocol Tests**
   ```rust
   #[test]
   fn test_start_recording() {
       let mut device = MockHiDocP1::new();
       assert!(device.send_command(Command::StartRecording).is_ok());
       assert_eq!(device.state, DeviceState::Recording);
   }
   ```

3. **Audio Transfer Tests**
   ```rust
   #[test]
   fn test_audio_streaming() {
       let mut device = MockHiDocP1::new();
       let chunk = device.read_audio_chunk()?;
       assert!(!chunk.is_empty());
       assert!(is_valid_pcm(&chunk));
   }
   ```

4. **Fallback Tests**
   ```rust
   #[test]
   fn test_mass_storage_import() {
       let temp_dir = tempfile::tempdir()?;
       // Create mock mount point with test audio file
       let importer = MassStorageImporter::new(temp_dir.path());
       let files = importer.scan_for_audio()?;
       assert_eq!(files.len(), 1);
   }
   ```

### Continuous Integration

1. **GitHub Actions Workflow**
   ```yaml
   name: USB Integration Tests
   on: [push, pull_request]
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Install libusb
           run: sudo apt-get install libusb-1.0-0-dev
         - name: Run tests
           run: cargo test --package hinotes-desktop --lib usb
   ```

2. **Mock-Only CI Tests**
   - All tests use mocked USB devices
   - No physical hardware required
   - Fast, reliable, repeatable

3. **Manual Device Testing**
   - Separate test suite for physical device
   - Run locally with `cargo test --features usb_device_tests --ignored`
   - Document in test plan

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Add `rusb` dependency
- [ ] Create `src/usb/` module structure
- [ ] Implement device enumeration
- [ ] Add VID/PID constants (placeholder)
- [ ] Create mock device for testing

### Phase 2: Protocol Discovery (Week 2-3)
- [ ] Capture USB packets from device
- [ ] Analyze packet structure
- [ ] Document command protocol
- [ ] Identify audio data format
- [ ] Update VID/PID with actual values

### Phase 3: Protocol Implementation (Week 4-5)
- [ ] Implement device communication layer
- [ ] Add command send/receive functions
- [ ] Implement audio streaming
- [ ] Add error recovery
- [ ] Integration with audio processor

### Phase 4: Mass Storage Fallback (Week 6)
- [ ] Detect mass storage mode
- [ ] Implement file system monitoring
- [ ] Add audio file import
- [ ] Auto-import workflow
- [ ] User preference for mode selection

### Phase 5: Testing & Polish (Week 7-8)
- [ ] Comprehensive test suite
- [ ] Cross-platform validation
- [ ] Performance optimization
- [ ] User documentation
- [ ] Device compatibility testing

## Security Considerations

1. **USB Device Validation**
   - Verify VID/PID before communication
   - Validate device descriptors
   - Implement timeout mechanisms

2. **Data Sanitization**
   - Validate audio data format
   - Check file sizes before import
   - Sanitize filenames from device

3. **Permission Handling**
   - Request USB permissions appropriately
   - Handle permission denials gracefully
   - Document required udev rules (Linux)

4. **Resource Management**
   - Proper device handle cleanup
   - Memory limits for audio buffers
   - Connection timeout handling

## Future Enhancements

1. **Multi-Device Support**
   - Support multiple HiDoc devices simultaneously
   - Device-specific configuration profiles

2. **Firmware Updates**
   - Device firmware version detection
   - OTA update support (if available)

3. **Advanced Features**
   - Real-time transcription during recording
   - Device status monitoring
   - Battery level indication
   - Storage capacity tracking

4. **Protocol Extensions**
   - Custom metadata fields
   - Enhanced audio quality settings
   - Device synchronization

## Resources

### Documentation
- [libusb API](https://libusb.info/)
- [rusb Documentation](https://docs.rs/rusb/)
- [USB Specification](https://www.usb.org/documents)
- [USB Audio Class Spec](https://www.usb.org/document-library/audio-device-class-spec-10)

### Tools
- [Wireshark USB Capture](https://wiki.wireshark.org/CaptureSetup/USB)
- [USBPcap](https://desowin.org/usbpcap/)
- [PacketLogger](https://developer.apple.com/bug-reporting/profiles-and-logs/)

### Community
- [libusb Mailing List](https://sourceforge.net/p/libusb/mailman/)
- [Rust USB Community](https://github.com/a1ien/rusb/discussions)

## Conclusion

This document provides a comprehensive roadmap for integrating the HiDoc P1 USB device. The dual approach (direct USB protocol + mass storage fallback) ensures maximum compatibility while pursuing optimal user experience through direct integration.

The mock-based testing strategy enables development and testing without requiring physical hardware access, ensuring continuous integration and rapid iteration.
