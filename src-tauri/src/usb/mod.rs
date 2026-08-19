//! USB Device Integration Module
//!
//! This module handles communication with the HiDoc P1 USB audio transcription device.
//! It provides both direct USB protocol communication and a mass storage fallback mode.

use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

pub mod detector;
pub mod mass_storage;
pub mod protocol;

// ============================================================================
// Device Constants
// ============================================================================

/// HiDoc P1 USB Device Identifiers
///
/// The HiDoc P1 exposes TWO USB interfaces:
/// 1. Audio interface (Actions Semiconductor chip): VID=0x10d6, PID=0xb00e
/// 2. Control/Storage interface (Solid State System): VID=0x1395, PID=0x005d
///
/// Discovered via: system_profiler SPUSBDataType on macOS
/// Serial Number: ACTIONS-BOS-002

/// Audio interface Vendor ID (Actions Semiconductor Co., Ltd.)
pub const HIDOC_P1_AUDIO_VID: u16 = 0x10d6;

/// Audio interface Product ID
pub const HIDOC_P1_AUDIO_PID: u16 = 0xb00e;

/// Control/Storage interface Vendor ID (Solid State System Co.,Ltd.)
pub const HIDOC_P1_CONTROL_VID: u16 = 0x1395;

/// Control/Storage interface Product ID
pub const HIDOC_P1_CONTROL_PID: u16 = 0x005d;

/// Device manufacturer string (for validation)
pub const HIDOC_MANUFACTURER: &str = "HiDock"; // Note: "HiDock" not "HiDoc"

/// Device product string (for validation)
pub const HIDOC_PRODUCT: &str = "P1";

/// USB communication timeout (milliseconds)
pub const USB_TIMEOUT_MS: u64 = 5000;

/// Maximum audio chunk size for bulk transfers (bytes)
pub const MAX_AUDIO_CHUNK_SIZE: usize = 16384; // 16KB chunks

// Backward compatibility aliases (deprecated)
#[deprecated(note = "Use HIDOC_P1_CONTROL_VID instead")]
pub const HIDOC_P1_VID: u16 = HIDOC_P1_CONTROL_VID;

#[deprecated(note = "Use HIDOC_P1_CONTROL_PID instead")]
pub const HIDOC_P1_PID: u16 = HIDOC_P1_CONTROL_PID;

/// Default audio sample rate (Hz)
/// Confirmed via macOS Audio MIDI Setup: 48kHz
pub const DEFAULT_SAMPLE_RATE: u32 = 48000; // 48kHz (confirmed)

/// Audio bit depth
/// Standard USB Audio Class specification
pub const AUDIO_BIT_DEPTH: u16 = 16; // 16-bit PCM

/// Number of input audio channels (recording)
pub const AUDIO_INPUT_CHANNELS: u16 = 1; // Mono input (confirmed)

/// Number of output audio channels (playback)
pub const AUDIO_OUTPUT_CHANNELS: u16 = 2; // Stereo output (confirmed)

// ============================================================================
// Device State
// ============================================================================

/// HiDoc P1 device state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceState {
    /// Device is disconnected
    Disconnected,
    /// Device is connected but not initialized
    Connected,
    /// Device is idle and ready for commands
    Idle,
    /// Device is actively recording
    Recording,
    /// Device is playing back audio
    Playing,
    /// Device is transferring data
    Transferring,
    /// Device is in an error state
    Error,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::Disconnected
    }
}

// ============================================================================
// Device Information
// ============================================================================

/// Information about a connected HiDoc P1 device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// USB Vendor ID
    pub vendor_id: u16,
    /// USB Product ID
    pub product_id: u16,
    /// Device serial number (if available)
    pub serial_number: Option<String>,
    /// Manufacturer string
    pub manufacturer: Option<String>,
    /// Product string
    pub product: Option<String>,
    /// USB bus number
    pub bus_number: u8,
    /// USB device address
    pub device_address: u8,
    /// Current device state
    pub state: DeviceState,
    /// Firmware version (if available)
    pub firmware_version: Option<String>,
    /// Available storage space (bytes, if mass storage mode)
    pub storage_available: Option<u64>,
    /// Total storage capacity (bytes, if mass storage mode)
    pub storage_total: Option<u64>,
}

impl DeviceInfo {
    /// Check if this is a HiDoc P1 device (either interface) based on VID/PID
    pub fn is_hidoc_p1(&self) -> bool {
        self.is_audio_interface() || self.is_control_interface()
    }

    /// Check if this is the audio interface (Actions Semiconductor)
    /// VID: 0x10d6, PID: 0xb00e
    pub fn is_audio_interface(&self) -> bool {
        self.vendor_id == HIDOC_P1_AUDIO_VID && self.product_id == HIDOC_P1_AUDIO_PID
    }

    /// Check if this is the control/storage interface (Solid State System)
    /// VID: 0x1395, PID: 0x005d
    pub fn is_control_interface(&self) -> bool {
        self.vendor_id == HIDOC_P1_CONTROL_VID && self.product_id == HIDOC_P1_CONTROL_PID
    }

    /// Validate device using both VID/PID and strings
    pub fn validate(&self) -> bool {
        if !self.is_hidoc_p1() {
            return false;
        }

        // Also check manufacturer/product strings if available
        if let Some(ref mfg) = self.manufacturer {
            // Accept manufacturer strings from either chip
            let valid_mfg = mfg.contains(HIDOC_MANUFACTURER)
                || mfg.contains("Actions Semiconductor")
                || mfg.contains("Solid State System");
            if !valid_mfg {
                warn!(
                    "Device VID {:04x} PID {:04x} matches but manufacturer '{}' unexpected",
                    self.vendor_id, self.product_id, mfg
                );
            }
        }

        if let Some(ref prod) = self.product {
            // Accept product name variations
            let valid_prod =
                prod.contains("HiDock") || prod.contains("HiDoc") || prod.contains(HIDOC_PRODUCT);
            if !valid_prod {
                warn!(
                    "Device VID {:04x} PID {:04x} matches but product '{}' unexpected",
                    self.vendor_id, self.product_id, prod
                );
            }
        }

        true
    }
}

// ============================================================================
// Device Mode
// ============================================================================

/// Operating mode for HiDoc P1 communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceMode {
    /// Direct USB protocol communication
    DirectUsb,
    /// Mass storage device mode (fallback)
    MassStorage,
    /// Automatic detection and mode selection
    Auto,
}

impl Default for DeviceMode {
    fn default() -> Self {
        Self::Auto
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// USB-specific error types
#[derive(Debug, thiserror::Error)]
pub enum UsbError {
    #[error("Device not found (VID: {vid:#06x}, PID: {pid:#06x})")]
    DeviceNotFound { vid: u16, pid: u16 },

    #[error("Failed to open device: {0}")]
    DeviceOpenFailed(String),

    #[error("Failed to claim interface {interface}: {reason}")]
    InterfaceClaimFailed { interface: u8, reason: String },

    #[error("USB transfer failed: {0}")]
    TransferFailed(String),

    #[error("USB transfer timed out after {timeout_ms}ms")]
    TransferTimeout { timeout_ms: u64 },

    #[error("Invalid device descriptor")]
    InvalidDescriptor,

    #[error("Device in invalid state: expected {expected:?}, got {actual:?}")]
    InvalidState {
        expected: DeviceState,
        actual: DeviceState,
    },

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Permission denied: USB access requires elevated privileges")]
    PermissionDenied,

    #[error("Unsupported device class: {0:#04x}")]
    UnsupportedDeviceClass(u8),
}

// ============================================================================
// Public API
// ============================================================================

/// Initialize the USB subsystem
///
/// Must be called before any USB operations.
pub fn init() -> Result<()> {
    info!("Initializing USB subsystem");

    // Test rusb context creation
    let _context = rusb::Context::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize USB context: {}", e))?;

    debug!("USB subsystem initialized successfully");
    Ok(())
}

/// Scan for connected HiDoc P1 devices
///
/// Returns a list of all detected devices matching the HiDoc P1 VID/PID.
pub fn scan_devices() -> Result<Vec<DeviceInfo>> {
    info!("Scanning for HiDoc P1 devices");

    let detector = detector::DeviceDetector::new()
        .map_err(|e| anyhow::anyhow!("Failed to create device detector: {}", e))?;

    let devices = detector
        .list_devices()
        .map_err(|e| anyhow::anyhow!("Failed to list devices: {}", e))?;

    if devices.is_empty() {
        debug!("No HiDoc P1 devices found");
    } else {
        info!("Found {} HiDoc P1 device(s)", devices.len());
    }

    Ok(devices)
}

/// Check if any HiDoc P1 device is connected
pub fn is_device_connected() -> bool {
    scan_devices()
        .map(|devices| !devices.is_empty())
        .unwrap_or(false)
}

// ============================================================================
// Testing Utilities
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_validation() {
        let mut info = DeviceInfo {
            vendor_id: HIDOC_P1_VID,
            product_id: HIDOC_P1_PID,
            serial_number: None,
            manufacturer: Some(HIDOC_MANUFACTURER.to_string()),
            product: Some(HIDOC_PRODUCT.to_string()),
            bus_number: 1,
            device_address: 2,
            state: DeviceState::Connected,
            firmware_version: None,
            storage_available: None,
            storage_total: None,
        };

        // Note: With placeholder VID/PID (0x0000), this will pass
        // but should be updated when real values are known
        assert!(info.is_hidoc_p1());
        assert!(info.validate());

        // Test with wrong VID
        info.vendor_id = 0xFFFF;
        assert!(!info.is_hidoc_p1());
        assert!(!info.validate());
    }

    #[test]
    fn test_device_state_default() {
        assert_eq!(DeviceState::default(), DeviceState::Disconnected);
    }

    #[test]
    fn test_device_mode_default() {
        assert_eq!(DeviceMode::default(), DeviceMode::Auto);
    }

    #[test]
    fn test_usb_error_display() {
        let err = UsbError::DeviceNotFound {
            vid: HIDOC_P1_VID,
            pid: HIDOC_P1_PID,
        };
        assert!(err.to_string().contains("Device not found"));

        let err = UsbError::TransferTimeout { timeout_ms: 5000 };
        assert!(err.to_string().contains("5000ms"));
    }

    #[test]
    #[ignore] // Requires USB hardware
    fn test_init() {
        // Should not panic with placeholder implementation
        assert!(init().is_ok());
    }

    #[test]
    #[ignore] // Requires USB hardware
    fn test_scan_devices_empty() {
        // Placeholder implementation returns empty vec
        let devices = scan_devices().expect("scan should not fail");
        assert!(devices.is_empty());
    }

    #[test]
    #[ignore] // Requires USB hardware
    fn test_is_device_connected() {
        // Should return false with placeholder implementation
        assert!(!is_device_connected());
    }
}
