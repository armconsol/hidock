//! Device Detection Module
//!
//! Handles USB device enumeration and HiDoc P1 detection.
//!
//! The HiDoc P1 exposes two USB interfaces:
//! - Audio interface: VID 0x10d6, PID 0xb00e (Actions Semiconductor)
//! - Control/Storage interface: VID 0x1395, PID 0x005d (Solid State System)

use super::{
    DeviceInfo, UsbError, HIDOC_P1_AUDIO_PID, HIDOC_P1_AUDIO_VID,
    HIDOC_P1_CONTROL_PID, HIDOC_P1_CONTROL_VID,
};
use anyhow::Result;
use log::info;

/// USB device detector
pub struct DeviceDetector {
    // TODO: Add rusb::Context when rusb is added
    // context: rusb::Context,
}

impl DeviceDetector {
    /// Create a new device detector
    pub fn new() -> Result<Self> {
        // TODO: Initialize rusb context
        // let context = rusb::Context::new()
        //     .map_err(|e| UsbError::DeviceOpenFailed(e.to_string()))?;

        Ok(Self {
            // context,
        })
    }

    /// Find the first HiDoc P1 device
    ///
    /// Returns the control interface (0x1395:0x005d) which is used for
    /// device management and file operations. Audio operations should use
    /// the audio interface (0x10d6:0xb00e).
    pub fn find_device(&self) -> Result<DeviceInfo, UsbError> {
        let devices = self.list_devices()?;

        devices
            .into_iter()
            .find(|d| d.is_hidoc_p1())
            .ok_or(UsbError::DeviceNotFound {
                vid: HIDOC_P1_CONTROL_VID,
                pid: HIDOC_P1_CONTROL_PID,
            })
    }

    /// Find the HiDoc P1 audio interface
    ///
    /// Returns the audio interface (0x10d6:0xb00e) which handles audio capture
    /// and playback at 48kHz, 16-bit PCM (mono input, stereo output).
    pub fn find_audio_device(&self) -> Result<DeviceInfo, UsbError> {
        let devices = self.list_devices()?;

        devices
            .into_iter()
            .find(|d| d.is_audio_interface())
            .ok_or(UsbError::DeviceNotFound {
                vid: HIDOC_P1_AUDIO_VID,
                pid: HIDOC_P1_AUDIO_PID,
            })
    }

    /// Find all HiDoc P1 interfaces (both audio and control)
    ///
    /// Returns a tuple of (audio_device, control_device) if both are found.
    pub fn find_all_interfaces(&self) -> Result<(DeviceInfo, DeviceInfo), UsbError> {
        let devices = self.list_devices()?;

        let audio = devices
            .iter()
            .find(|d| d.is_audio_interface())
            .cloned()
            .ok_or(UsbError::DeviceNotFound {
                vid: HIDOC_P1_AUDIO_VID,
                pid: HIDOC_P1_AUDIO_PID,
            })?;

        let control = devices
            .into_iter()
            .find(|d| d.is_control_interface())
            .ok_or(UsbError::DeviceNotFound {
                vid: HIDOC_P1_CONTROL_VID,
                pid: HIDOC_P1_CONTROL_PID,
            })?;

        Ok((audio, control))
    }

    /// List all connected HiDoc P1 devices
    ///
    /// This scans for both interfaces:
    /// - Audio interface: VID 0x10d6, PID 0xb00e
    /// - Control/Storage interface: VID 0x1395, PID 0x005d
    ///
    /// Also validates by manufacturer name "HiDock" for additional confidence.
    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>, UsbError> {
        info!("Enumerating USB devices for HiDoc P1 (checking both interfaces)");

        // TODO: Implement with rusb
        /*
        let devices = self.context.devices()
            .map_err(|e| UsbError::DeviceOpenFailed(e.to_string()))?;

        let mut hidoc_devices = Vec::new();

        for device in devices.iter() {
            let desc = device.device_descriptor()
                .map_err(|_| UsbError::InvalidDescriptor)?;

            let vid = desc.vendor_id();
            let pid = desc.product_id();

            // Check for either audio or control interface
            let is_audio = vid == HIDOC_P1_AUDIO_VID && pid == HIDOC_P1_AUDIO_PID;
            let is_control = vid == HIDOC_P1_CONTROL_VID && pid == HIDOC_P1_CONTROL_PID;

            if is_audio || is_control {
                match self.get_device_info(&device, &desc) {
                    Ok(info) => {
                        // Validate manufacturer name for additional confidence
                        if let Some(ref mfg) = info.manufacturer {
                            if !mfg.contains(HIDOC_MANUFACTURER) {
                                warn!(
                                    "Device VID:PID {:04x}:{:04x} matches but manufacturer '{}' does not contain '{}'",
                                    vid, pid, mfg, HIDOC_MANUFACTURER
                                );
                            }
                        }

                        info!(
                            "Found HiDoc P1 {} interface: VID {:04x} PID {:04x} Serial: {}",
                            if is_audio { "audio" } else { "control" },
                            vid,
                            pid,
                            info.serial_number.as_deref().unwrap_or("N/A")
                        );
                        hidoc_devices.push(info);
                    }
                    Err(e) => {
                        warn!("Failed to get device info for {:04x}:{:04x}: {}", vid, pid, e);
                    }
                }
            }
        }

        if hidoc_devices.is_empty() {
            info!("No HiDoc P1 devices found");
        } else {
            info!("Found {} HiDoc P1 interface(s)", hidoc_devices.len());
        }

        Ok(hidoc_devices)
        */

        // Placeholder - returns empty until rusb is integrated
        info!("USB enumeration not yet implemented (rusb integration pending)");
        Ok(Vec::new())
    }

    /// Extract device information from a USB device
    /*
    fn get_device_info(
        &self,
        device: &rusb::Device<rusb::Context>,
        desc: &rusb::DeviceDescriptor,
    ) -> Result<DeviceInfo, UsbError> {
        let handle = device.open()
            .map_err(|e| UsbError::DeviceOpenFailed(e.to_string()))?;

        let timeout = Duration::from_millis(USB_TIMEOUT_MS);

        // Read string descriptors
        let manufacturer = desc.manufacturer_string_index()
            .and_then(|idx| handle.read_manufacturer_string(desc, &timeout).ok());

        let product = desc.product_string_index()
            .and_then(|idx| handle.read_product_string(desc, &timeout).ok());

        let serial_number = desc.serial_number_string_index()
            .and_then(|idx| handle.read_serial_number_string(desc, &timeout).ok());

        Ok(DeviceInfo {
            vendor_id: desc.vendor_id(),
            product_id: desc.product_id(),
            serial_number,
            manufacturer,
            product,
            bus_number: device.bus_number(),
            device_address: device.address(),
            state: DeviceState::Connected,
            firmware_version: None,
            storage_available: None,
            storage_total: None,
        })
    }
    */

    /// Check if device supports mass storage class
    pub fn is_mass_storage_device(&self, _info: &DeviceInfo) -> Result<bool, UsbError> {
        // TODO: Implement with rusb
        /*
        let device = self.find_device_by_address(info.bus_number, info.device_address)?;
        let config = device.active_config_descriptor()
            .map_err(|_| UsbError::InvalidDescriptor)?;

        for interface in config.interfaces() {
            for desc in interface.descriptors() {
                // Mass Storage Class = 0x08
                if desc.class_code() == 0x08 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
        */

        // Placeholder
        Ok(false)
    }
}

impl Default for DeviceDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create device detector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = DeviceDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_find_device_not_found() {
        let detector = DeviceDetector::new().unwrap();
        let result = detector.find_device();

        // Should fail when no device is connected (placeholder implementation)
        // Real implementation searches for:
        //   - Control Interface: VID=0x1395, PID=0x005d (Solid State System)
        //   - Audio Interface: VID=0x10d6, PID=0xb00e (Actions Semiconductor)
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, UsbError::DeviceNotFound { .. }));
        }
    }

    #[test]
    fn test_list_devices_empty() {
        let detector = DeviceDetector::new().unwrap();
        let devices = detector.list_devices().unwrap();
        assert!(devices.is_empty());
    }
}
