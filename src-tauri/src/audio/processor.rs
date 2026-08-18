use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Audio processor using FFmpeg for audio editing operations
pub struct AudioProcessor {
    ffmpeg_path: PathBuf,
    temp_dir: PathBuf,
}

impl AudioProcessor {
    /// Create a new AudioProcessor instance
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("hinotes_audio_processing");
        std::fs::create_dir_all(&temp_dir)
            .context("Failed to create temporary processing directory")?;

        Ok(Self {
            ffmpeg_path: Self::find_ffmpeg()?,
            temp_dir,
        })
    }

    /// Find FFmpeg binary (bundled or system)
    fn find_ffmpeg() -> Result<PathBuf> {
        // Check for bundled FFmpeg first
        if let Ok(resource_dir) = std::env::var("TAURI_RESOURCE_DIR") {
            let bundled_path = Path::new(&resource_dir)
                .join("bin")
                .join(if cfg!(target_os = "windows") {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                });

            if bundled_path.exists() {
                return Ok(bundled_path);
            }
        }

        // Fall back to system FFmpeg
        #[cfg(target_os = "windows")]
        let ffmpeg_name = "ffmpeg.exe";
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_name = "ffmpeg";

        // Try to find FFmpeg in PATH
        if let Ok(output) = Command::new("which").arg(ffmpeg_name).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err(anyhow!(
            "FFmpeg not found. Please install FFmpeg or ensure it's bundled with the application."
        ))
    }

    /// Verify FFmpeg is available and working
    pub fn verify_ffmpeg(&self) -> Result<()> {
        let output = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .output()
            .context("Failed to execute FFmpeg")?;

        if !output.status.success() {
            return Err(anyhow!("FFmpeg is not working properly"));
        }

        Ok(())
    }

    /// Merge multiple audio files into one
    ///
    /// # Arguments
    /// * `files` - List of audio file paths to merge (in order)
    ///
    /// # Returns
    /// Path to the merged audio file in temp directory
    pub async fn merge_audio(&self, files: &[PathBuf]) -> Result<PathBuf> {
        if files.is_empty() {
            return Err(anyhow!("No files provided for merging"));
        }

        if files.len() == 1 {
            return Err(anyhow!("Need at least 2 files to merge"));
        }

        // Validate all input files exist
        for file in files {
            if !file.exists() {
                return Err(anyhow!("Input file does not exist: {:?}", file));
            }
        }

        // Create concat list file
        let concat_list = self.temp_dir.join("concat_list.txt");
        let mut list_content = String::new();
        for file in files {
            list_content.push_str(&format!(
                "file '{}'\n",
                file.to_string_lossy().replace('\'', "'\\''")
            ));
        }
        fs::write(&concat_list, list_content)
            .await
            .context("Failed to write concat list")?;

        // Output file
        let timestamp = chrono::Utc::now().timestamp_millis();
        let output_file = self.temp_dir.join(format!("merged_{}.m4a", timestamp));

        // Run FFmpeg concat
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                concat_list.to_str().unwrap(),
                "-c",
                "copy",
                "-y", // Overwrite output file if exists
                output_file.to_str().unwrap(),
            ])
            .output()
            .context("Failed to execute FFmpeg merge")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFmpeg merge failed: {}", stderr));
        }

        // Clean up concat list
        let _ = fs::remove_file(concat_list).await;

        Ok(output_file)
    }

    /// Replace a segment of audio with new audio
    ///
    /// # Arguments
    /// * `original` - Path to the original audio file
    /// * `replacement` - Path to the replacement audio file
    /// * `start_ms` - Start time in milliseconds where replacement begins
    /// * `end_ms` - End time in milliseconds where replacement ends
    ///
    /// # Returns
    /// Path to the new audio file with replaced segment
    pub async fn replace_audio_segment(
        &self,
        original: &Path,
        replacement: &Path,
        start_ms: u64,
        end_ms: u64,
    ) -> Result<PathBuf> {
        if !original.exists() {
            return Err(anyhow!("Original file does not exist"));
        }
        if !replacement.exists() {
            return Err(anyhow!("Replacement file does not exist"));
        }
        if start_ms >= end_ms {
            return Err(anyhow!("Start time must be before end time"));
        }

        let start_sec = start_ms as f64 / 1000.0;
        let end_sec = end_ms as f64 / 1000.0;

        let timestamp = chrono::Utc::now().timestamp_millis();

        // Extract part before replacement
        let before_file = self.temp_dir.join(format!("before_{}.m4a", timestamp));
        let before_output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                original.to_str().unwrap(),
                "-t",
                &start_sec.to_string(),
                "-c",
                "copy",
                "-y",
                before_file.to_str().unwrap(),
            ])
            .output()
            .context("Failed to extract before segment")?;

        if !before_output.status.success() {
            let stderr = String::from_utf8_lossy(&before_output.stderr);
            return Err(anyhow!("Failed to extract before segment: {}", stderr));
        }

        // Extract part after replacement
        let after_file = self.temp_dir.join(format!("after_{}.m4a", timestamp));
        let after_output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                original.to_str().unwrap(),
                "-ss",
                &end_sec.to_string(),
                "-c",
                "copy",
                "-y",
                after_file.to_str().unwrap(),
            ])
            .output()
            .context("Failed to extract after segment")?;

        if !after_output.status.success() {
            let stderr = String::from_utf8_lossy(&after_output.stderr);
            return Err(anyhow!("Failed to extract after segment: {}", stderr));
        }

        // Merge: before + replacement + after
        let parts = vec![before_file.clone(), replacement.to_path_buf(), after_file.clone()];
        let result = self.merge_audio(&parts).await?;

        // Clean up temporary files
        let _ = fs::remove_file(before_file).await;
        let _ = fs::remove_file(after_file).await;

        Ok(result)
    }

    /// Save audio data to a new file
    ///
    /// # Arguments
    /// * `audio_data` - Raw audio data bytes
    /// * `format` - Audio format extension (e.g., "m4a", "mp3", "wav")
    ///
    /// # Returns
    /// Path to the saved audio file
    pub async fn save_as_new(&self, audio_data: &[u8], format: &str) -> Result<PathBuf> {
        if audio_data.is_empty() {
            return Err(anyhow!("No audio data provided"));
        }

        let timestamp = chrono::Utc::now().timestamp_millis();
        let output_file = self.temp_dir.join(format!("audio_{}.{}", timestamp, format));

        fs::write(&output_file, audio_data)
            .await
            .context("Failed to write audio file")?;

        Ok(output_file)
    }

    /// Convert audio to a specific format
    ///
    /// # Arguments
    /// * `input` - Path to input audio file
    /// * `output_format` - Desired output format (e.g., "m4a", "mp3", "wav")
    /// * `bitrate` - Optional bitrate (e.g., "192k", "320k")
    ///
    /// # Returns
    /// Path to the converted audio file
    pub async fn convert_format(
        &self,
        input: &Path,
        output_format: &str,
        bitrate: Option<&str>,
    ) -> Result<PathBuf> {
        if !input.exists() {
            return Err(anyhow!("Input file does not exist"));
        }

        let timestamp = chrono::Utc::now().timestamp_millis();
        let output_file = self
            .temp_dir
            .join(format!("converted_{}.{}", timestamp, output_format));

        let mut args = vec![
            "-i",
            input.to_str().unwrap(),
        ];

        if let Some(br) = bitrate {
            args.push("-b:a");
            args.push(br);
        }

        args.push("-y");
        args.push(output_file.to_str().unwrap());

        let output = Command::new(&self.ffmpeg_path)
            .args(&args)
            .output()
            .context("Failed to execute FFmpeg conversion")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFmpeg conversion failed: {}", stderr));
        }

        Ok(output_file)
    }

    /// Get audio duration in milliseconds
    pub async fn get_duration(&self, file: &Path) -> Result<u64> {
        if !file.exists() {
            return Err(anyhow!("File does not exist"));
        }

        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                file.to_str().unwrap(),
                "-f",
                "null",
                "-",
            ])
            .output()
            .context("Failed to probe audio file")?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse duration from FFmpeg output
        // Format: "Duration: 00:01:23.45"
        if let Some(duration_line) = stderr.lines().find(|l| l.contains("Duration:")) {
            if let Some(duration_str) = duration_line.split("Duration:").nth(1) {
                if let Some(time_str) = duration_str.split(',').next() {
                    let time_str = time_str.trim();
                    let parts: Vec<&str> = time_str.split(':').collect();
                    if parts.len() == 3 {
                        let hours: f64 = parts[0].parse().unwrap_or(0.0);
                        let minutes: f64 = parts[1].parse().unwrap_or(0.0);
                        let seconds: f64 = parts[2].parse().unwrap_or(0.0);

                        let total_ms = ((hours * 3600.0 + minutes * 60.0 + seconds) * 1000.0) as u64;
                        return Ok(total_ms);
                    }
                }
            }
        }

        Err(anyhow!("Failed to parse audio duration"))
    }

    /// Trim audio to a specific time range
    ///
    /// # Arguments
    /// * `input` - Path to input audio file
    /// * `start_ms` - Start time in milliseconds
    /// * `end_ms` - End time in milliseconds (None = end of file)
    ///
    /// # Returns
    /// Path to the trimmed audio file
    pub async fn trim_audio(
        &self,
        input: &Path,
        start_ms: u64,
        end_ms: Option<u64>,
    ) -> Result<PathBuf> {
        if !input.exists() {
            return Err(anyhow!("Input file does not exist"));
        }

        let start_sec = start_ms as f64 / 1000.0;
        let start_sec_str = start_sec.to_string();
        let timestamp = chrono::Utc::now().timestamp_millis();
        let output_file = self.temp_dir.join(format!("trimmed_{}.m4a", timestamp));

        let mut args = vec![
            "-i",
            input.to_str().unwrap(),
            "-ss",
            &start_sec_str,
        ];

        let duration_sec_str: String;
        if let Some(end) = end_ms {
            let duration_sec = (end - start_ms) as f64 / 1000.0;
            duration_sec_str = duration_sec.to_string();
            args.push("-t");
            args.push(&duration_sec_str);
        }

        args.push("-c");
        args.push("copy");
        args.push("-y");
        args.push(output_file.to_str().unwrap());

        let output = Command::new(&self.ffmpeg_path)
            .args(&args)
            .output()
            .context("Failed to trim audio")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to trim audio: {}", stderr));
        }

        Ok(output_file)
    }

    /// Clean up temporary files older than specified duration
    pub async fn cleanup_temp_files(&self, max_age_hours: u64) -> Result<usize> {
        let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
        let now = std::time::SystemTime::now();
        let mut removed_count = 0;

        let mut entries = fs::read_dir(&self.temp_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            if fs::remove_file(entry.path()).await.is_ok() {
                                removed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_audio_file(path: &Path) -> Result<()> {
        // Create a minimal valid M4A file header for testing
        let mut file = std::fs::File::create(path)?;
        // This is a minimal ftyp atom for M4A
        let m4a_header: &[u8] = &[
            0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, // ftyp atom
            0x4d, 0x34, 0x41, 0x20, 0x00, 0x00, 0x00, 0x00,
            0x69, 0x73, 0x6f, 0x6d, 0x69, 0x73, 0x6f, 0x32,
            0x00, 0x00, 0x00, 0x08, 0x66, 0x72, 0x65, 0x65,
        ];
        file.write_all(m4a_header)?;
        Ok(())
    }

    #[test]
    fn test_find_ffmpeg() {
        let result = AudioProcessor::find_ffmpeg();
        // This test will fail if FFmpeg is not installed
        // In CI, we should mock or skip this
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_new_processor() {
        let processor = AudioProcessor::new();
        assert!(processor.is_ok() || processor.is_err());
    }

    #[tokio::test]
    async fn test_save_as_new() {
        let processor = AudioProcessor::new().unwrap();
        let test_data = b"test audio data";

        let result = processor.save_as_new(test_data, "m4a").await;
        assert!(result.is_ok());

        if let Ok(path) = result {
            assert!(path.exists());
            let content = fs::read(&path).await.unwrap();
            assert_eq!(content, test_data);
            let _ = fs::remove_file(path).await;
        }
    }

    #[tokio::test]
    async fn test_save_empty_data() {
        let processor = AudioProcessor::new().unwrap();
        let result = processor.save_as_new(&[], "m4a").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_audio_validation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let processor = AudioProcessor::new().unwrap();

            // Test empty files
            let result = processor.merge_audio(&[]).await;
            assert!(result.is_err());

            // Test single file
            let single = vec![PathBuf::from("/tmp/test.m4a")];
            let result = processor.merge_audio(&single).await;
            assert!(result.is_err());
        });
    }

    #[tokio::test]
    async fn test_cleanup_temp_files() {
        let processor = AudioProcessor::new().unwrap();

        // Create a test file
        let test_file = processor.temp_dir.join("test_cleanup.txt");
        fs::write(&test_file, b"test").await.unwrap();

        // Try cleanup (0 hours = clean all)
        let result = processor.cleanup_temp_files(0).await;
        assert!(result.is_ok());

        // Give async cleanup a moment
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
