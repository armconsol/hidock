//! Mass Storage Fallback Implementation
//!
//! Provides file-based integration when direct USB protocol is unavailable
//! or the device operates in mass storage mode.

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Constants
// ============================================================================

/// Expected volume name for HiDoc P1 mass storage
const VOLUME_NAME: &str = "HIDOC_P1";

/// Supported audio file extensions
const AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "m4a", "aac"];

/// Default mount point patterns per platform
#[cfg(target_os = "macos")]
const MOUNT_POINT_PATTERNS: &[&str] = &["/Volumes/HIDOC*", "/Volumes/P1*"];

#[cfg(target_os = "linux")]
const MOUNT_POINT_PATTERNS: &[&str] = &[
    "/media/${USER}/HIDOC*",
    "/media/${USER}/P1*",
    "/mnt/HIDOC*",
    "/run/media/${USER}/HIDOC*",
];

#[cfg(target_os = "windows")]
const MOUNT_POINT_PATTERNS: &[&str] = &["?:\\"];

// ============================================================================
// Types
// ============================================================================

/// Audio file metadata from mass storage device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFileInfo {
    /// Full path to the audio file
    pub path: PathBuf,
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// File creation time (if available)
    pub created: Option<std::time::SystemTime>,
    /// File modification time
    pub modified: std::time::SystemTime,
    /// Audio format (extension)
    pub format: String,
}

/// Mass storage device mount information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    /// Mount point path
    pub path: PathBuf,
    /// Volume label
    pub label: Option<String>,
    /// Available space (bytes)
    pub available: u64,
    /// Total capacity (bytes)
    pub total: u64,
}

// ============================================================================
// Mass Storage Importer
// ============================================================================

/// Mass storage audio file importer
pub struct MassStorageImporter {
    /// Mount point path
    mount_point: PathBuf,
}

impl MassStorageImporter {
    /// Create a new importer for the given mount point
    pub fn new<P: AsRef<Path>>(mount_point: P) -> Self {
        Self {
            mount_point: mount_point.as_ref().to_path_buf(),
        }
    }

    /// Try to detect HiDoc P1 mass storage mount point
    pub fn detect_mount_point() -> Option<PathBuf> {
        info!("Detecting HiDoc P1 mass storage mount point");

        #[cfg(target_os = "macos")]
        {
            Self::detect_mount_point_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::detect_mount_point_linux()
        }

        #[cfg(target_os = "windows")]
        {
            Self::detect_mount_point_windows()
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_mount_point_macos() -> Option<PathBuf> {
        // Check /Volumes for HiDoc device
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains("HIDOC") || name.contains("P1") {
                        info!("Found potential mount point: {:?}", path);
                        return Some(path);
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn detect_mount_point_linux() -> Option<PathBuf> {
        // Check common mount locations
        let username = std::env::var("USER").ok()?;
        let media_paths = vec![
            format!("/media/{}/", username),
            format!("/run/media/{}/", username),
            "/mnt/".to_string(),
        ];

        for base_path in media_paths {
            if let Ok(entries) = fs::read_dir(&base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.contains("HIDOC") || name.contains("P1") {
                            info!("Found potential mount point: {:?}", path);
                            return Some(path);
                        }
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn detect_mount_point_windows() -> Option<PathBuf> {
        // Check all drive letters
        for letter in 'D'..='Z' {
            let path = PathBuf::from(format!("{}:\\", letter));
            if path.exists() {
                // Try to read volume label
                // This is a simplified check; actual Windows volume label reading
                // would require Windows API calls
                if let Ok(entries) = fs::read_dir(&path) {
                    // Just check if drive is readable
                    info!("Found readable drive: {:?}", path);
                    // TODO: Check volume label via Windows API
                    // For now, just return first readable drive after C:
                    if letter > 'C' {
                        return Some(path);
                    }
                }
            }
        }
        None
    }

    /// Get mount information
    pub fn get_mount_info(&self) -> Result<MountInfo> {
        let _metadata =
            fs::metadata(&self.mount_point).context("Failed to read mount point metadata")?;

        // Get filesystem stats (platform-specific)
        let (available, total) = self.get_filesystem_stats()?;

        Ok(MountInfo {
            path: self.mount_point.clone(),
            label: self.get_volume_label(),
            available,
            total,
        })
    }

    #[cfg(unix)]
    fn get_filesystem_stats(&self) -> Result<(u64, u64)> {
        // Use statvfs for accurate filesystem information
        // This is a simplified version; consider using nix crate for full statvfs
        let _metadata = fs::metadata(&self.mount_point)?;

        // Placeholder values
        // TODO: Use nix::sys::statvfs::statvfs for actual values
        Ok((0, 0))
    }

    #[cfg(windows)]
    fn get_filesystem_stats(&self) -> Result<(u64, u64)> {
        // TODO: Use Windows API (GetDiskFreeSpaceEx) for actual values
        Ok((0, 0))
    }

    fn get_volume_label(&self) -> Option<String> {
        // Simplified volume label extraction
        self.mount_point
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    /// Scan for audio files
    pub fn scan_for_audio(&self) -> Result<Vec<AudioFileInfo>> {
        info!("Scanning {:?} for audio files", self.mount_point);

        let mut audio_files = Vec::new();

        self.scan_directory(&self.mount_point, &mut audio_files)?;

        info!("Found {} audio file(s)", audio_files.len());
        Ok(audio_files)
    }

    /// Recursively scan directory for audio files
    fn scan_directory(&self, dir: &Path, files: &mut Vec<AudioFileInfo>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries =
            fs::read_dir(dir).with_context(|| format!("Failed to read directory: {:?}", dir))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recurse into subdirectories
                self.scan_directory(&path, files)?;
            } else if let Some(info) = self.check_audio_file(&path) {
                files.push(info);
            }
        }

        Ok(())
    }

    /// Check if file is a supported audio file and extract metadata
    fn check_audio_file(&self, path: &Path) -> Option<AudioFileInfo> {
        let extension = path.extension()?.to_str()?.to_lowercase();

        if !AUDIO_EXTENSIONS.contains(&extension.as_str()) {
            return None;
        }

        let metadata = fs::metadata(path).ok()?;

        Some(AudioFileInfo {
            path: path.to_path_buf(),
            name: path.file_name()?.to_str()?.to_string(),
            size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok()?,
            format: extension,
        })
    }

    /// Copy audio file to local storage
    pub fn import_audio_file(&self, file: &AudioFileInfo, dest_dir: &Path) -> Result<PathBuf> {
        info!("Importing audio file: {}", file.name);

        // Ensure destination directory exists
        fs::create_dir_all(dest_dir).context("Failed to create destination directory")?;

        let dest_path = dest_dir.join(&file.name);

        // Copy file
        fs::copy(&file.path, &dest_path)
            .with_context(|| format!("Failed to copy {} to {:?}", file.name, dest_path))?;

        info!("Audio file imported to {:?}", dest_path);
        Ok(dest_path)
    }

    /// Delete audio file from device
    pub fn delete_audio_file(&self, file: &AudioFileInfo) -> Result<()> {
        info!("Deleting audio file: {}", file.name);

        fs::remove_file(&file.path).with_context(|| format!("Failed to delete {}", file.name))?;

        Ok(())
    }
}

// ============================================================================
// File System Monitoring
// ============================================================================

/// Monitor for new audio files (requires notify crate)
pub struct AudioFileMonitor {
    mount_point: PathBuf,
}

impl AudioFileMonitor {
    pub fn new<P: AsRef<Path>>(mount_point: P) -> Self {
        Self {
            mount_point: mount_point.as_ref().to_path_buf(),
        }
    }

    /// Start monitoring for new audio files
    ///
    /// TODO: Implement with notify crate for real-time file system events
    pub fn start_monitoring<F>(&self, _callback: F) -> Result<()>
    where
        F: Fn(AudioFileInfo) + Send + 'static,
    {
        info!("Starting audio file monitoring on {:?}", self.mount_point);

        // TODO: Implement with notify crate
        /*
        use notify::{Watcher, RecursiveMode, watcher};
        use std::sync::mpsc::channel;

        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(2))?;

        watcher.watch(&self.mount_point, RecursiveMode::Recursive)?;

        loop {
            match rx.recv() {
                Ok(event) => {
                    // Handle file system event
                    // Check if new audio file
                    // Call callback
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
        */

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_audio_extension_check() {
        let extensions = ["wav", "mp3", "m4a", "aac"];
        for ext in extensions {
            assert!(AUDIO_EXTENSIONS.contains(&ext));
        }
        assert!(!AUDIO_EXTENSIONS.contains(&"txt"));
    }

    #[test]
    fn test_mass_storage_importer_creation() {
        let importer = MassStorageImporter::new("/tmp");
        assert_eq!(importer.mount_point, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_scan_for_audio_empty_dir() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let importer = MassStorageImporter::new(temp_dir.path());
        let files = importer.scan_for_audio()?;
        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn test_scan_for_audio_with_files() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        // Create test audio file
        let audio_path = temp_dir.path().join("test.wav");
        File::create(&audio_path)?.write_all(b"fake wav data")?;

        // Create non-audio file
        let text_path = temp_dir.path().join("readme.txt");
        File::create(&text_path)?.write_all(b"text")?;

        let importer = MassStorageImporter::new(temp_dir.path());
        let files = importer.scan_for_audio()?;

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.wav");
        assert_eq!(files[0].format, "wav");

        Ok(())
    }

    #[test]
    fn test_import_audio_file() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&source_dir)?;

        // Create test audio file
        let audio_path = source_dir.join("test.wav");
        File::create(&audio_path)?.write_all(b"fake wav data")?;

        let importer = MassStorageImporter::new(&source_dir);
        let files = importer.scan_for_audio()?;
        assert_eq!(files.len(), 1);

        // Import the file
        let imported_path = importer.import_audio_file(&files[0], &dest_dir)?;

        assert!(imported_path.exists());
        assert_eq!(fs::read(&imported_path)?, b"fake wav data");

        Ok(())
    }
}
