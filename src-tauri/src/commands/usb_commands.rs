//! USB Device Management Commands
//!
//! Tauri commands for USB device detection, enumeration, and control.

use crate::usb::{self, DeviceInfo};
use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbScanResult {
    /// List of detected devices
    pub devices: Vec<DeviceInfo>,
    /// Number of audio interfaces found
    pub audio_interfaces: usize,
    /// Number of control interfaces found
    pub control_interfaces: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassStorageScanResult {
    /// Mount point path if found
    pub mount_point: Option<PathBuf>,
    /// List of audio files found
    pub audio_files: Vec<AudioFileMetadata>,
    /// Available storage space (bytes)
    pub available_space: Option<u64>,
    /// Total storage capacity (bytes)
    pub total_space: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFileMetadata {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub format: String,
    pub created: Option<i64>, // Unix timestamp
    pub modified: i64,        // Unix timestamp
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Initialize the USB subsystem
///
/// Must be called before any other USB operations.
#[tauri::command]
pub async fn usb_init() -> Result<(), String> {
    info!("Initializing USB subsystem");

    usb::init().map_err(|e| format!("Failed to initialize USB: {}", e))
}

/// Scan for connected HiDoc P1 devices
///
/// Returns information about all detected HiDoc P1 interfaces (audio and control).
#[tauri::command]
pub async fn usb_scan_devices() -> Result<UsbScanResult, String> {
    info!("Scanning for USB devices");

    let devices = usb::scan_devices().map_err(|e| format!("Failed to scan devices: {}", e))?;

    let audio_interfaces = devices.iter().filter(|d| d.is_audio_interface()).count();
    let control_interfaces = devices.iter().filter(|d| d.is_control_interface()).count();

    Ok(UsbScanResult {
        devices,
        audio_interfaces,
        control_interfaces,
    })
}

/// Check if any HiDoc P1 device is connected
#[tauri::command]
pub async fn usb_is_device_connected() -> Result<bool, String> {
    Ok(usb::is_device_connected())
}

/// Detect HiDoc P1 mass storage mount point and scan for audio files
///
/// This is a fallback mode when direct USB protocol access is not available.
#[tauri::command]
pub async fn usb_scan_mass_storage() -> Result<MassStorageScanResult, String> {
    info!("Scanning for HiDoc P1 mass storage mount point");

    let mount_point = usb::mass_storage::MassStorageImporter::detect_mount_point();

    if let Some(ref path) = mount_point {
        info!("Found mount point: {:?}", path);

        let importer = usb::mass_storage::MassStorageImporter::new(path);

        // Get mount info
        let mount_info = importer.get_mount_info().map_err(|e| {
            error!("Failed to get mount info: {}", e);
            format!("Failed to get mount info: {}", e)
        })?;

        // Scan for audio files
        let audio_files_raw = importer.scan_for_audio().map_err(|e| {
            error!("Failed to scan for audio files: {}", e);
            format!("Failed to scan for audio files: {}", e)
        })?;

        // Convert to serializable format
        let audio_files = audio_files_raw
            .into_iter()
            .map(|f| AudioFileMetadata {
                path: f.path,
                name: f.name,
                size: f.size,
                format: f.format,
                created: f.created.and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs() as i64)
                }),
                modified: f
                    .modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            })
            .collect();

        Ok(MassStorageScanResult {
            mount_point: Some(path.clone()),
            audio_files,
            available_space: Some(mount_info.available),
            total_space: Some(mount_info.total),
        })
    } else {
        info!("No mass storage mount point found");
        Ok(MassStorageScanResult {
            mount_point: None,
            audio_files: Vec::new(),
            available_space: None,
            total_space: None,
        })
    }
}

/// Import audio file from mass storage device to local directory
#[tauri::command]
pub async fn usb_import_audio_file(
    file_path: PathBuf,
    dest_dir: PathBuf,
) -> Result<PathBuf, String> {
    info!("Importing audio file: {:?} to {:?}", file_path, dest_dir);

    // Find mount point
    let mount_point = usb::mass_storage::MassStorageImporter::detect_mount_point()
        .ok_or_else(|| "No mass storage device found".to_string())?;

    let importer = usb::mass_storage::MassStorageImporter::new(mount_point);

    // Scan for audio files to get metadata
    let audio_files = importer
        .scan_for_audio()
        .map_err(|e| format!("Failed to scan for audio files: {}", e))?;

    // Find the file to import
    let file_info = audio_files
        .into_iter()
        .find(|f| f.path == file_path)
        .ok_or_else(|| format!("File not found: {:?}", file_path))?;

    // Import the file
    importer
        .import_audio_file(&file_info, &dest_dir)
        .map_err(|e| format!("Failed to import file: {}", e))
}

/// Delete audio file from mass storage device
#[tauri::command]
pub async fn usb_delete_audio_file(file_path: PathBuf) -> Result<(), String> {
    info!("Deleting audio file: {:?}", file_path);

    // Find mount point
    let mount_point = usb::mass_storage::MassStorageImporter::detect_mount_point()
        .ok_or_else(|| "No mass storage device found".to_string())?;

    let importer = usb::mass_storage::MassStorageImporter::new(mount_point);

    // Scan for audio files to get metadata
    let audio_files = importer
        .scan_for_audio()
        .map_err(|e| format!("Failed to scan for audio files: {}", e))?;

    // Find the file to delete
    let file_info = audio_files
        .into_iter()
        .find(|f| f.path == file_path)
        .ok_or_else(|| format!("File not found: {:?}", file_path))?;

    // Delete the file
    importer
        .delete_audio_file(&file_info)
        .map_err(|e| format!("Failed to delete file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_usb_init() {
        // Should not panic
        let result = usb_init().await;
        // May fail if USB subsystem unavailable, but shouldn't panic
        println!("USB init result: {:?}", result);
    }

    #[tokio::test]
    async fn test_usb_scan_devices() {
        // Should not panic even if no devices found
        let result = usb_scan_devices().await;
        match result {
            Ok(scan_result) => {
                println!("Found {} devices", scan_result.devices.len());
                println!(
                    "Audio interfaces: {}, Control interfaces: {}",
                    scan_result.audio_interfaces, scan_result.control_interfaces
                );
            }
            Err(e) => {
                println!("Scan failed (expected if no USB access): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_usb_is_device_connected() {
        let result = usb_is_device_connected().await;
        match result {
            Ok(connected) => println!("Device connected: {}", connected),
            Err(e) => println!("Check failed: {}", e),
        }
    }
}
