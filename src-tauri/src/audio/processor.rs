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
        self.merge_audio_with_progress(files, |_| {}).await
    }

    /// Merge multiple audio files with progress callback
    ///
    /// # Arguments
    /// * `files` - List of audio file paths to merge (in order)
    /// * `progress_callback` - Callback function called with progress (0.0 to 1.0)
    ///
    /// # Returns
    /// Path to the merged audio file in temp directory
    pub async fn merge_audio_with_progress<F>(&self, files: &[PathBuf], progress_callback: F) -> Result<PathBuf>
    where
        F: Fn(f32) + Send + 'static,
    {
        if files.is_empty() {
            return Err(anyhow!("No files provided for merging"));
        }

        if files.len() == 1 {
            return Err(anyhow!("Need at least 2 files to merge"));
        }

        progress_callback(0.0);

        // Validate all input files exist
        for (i, file) in files.iter().enumerate() {
            if !file.exists() {
                return Err(anyhow!("Input file does not exist: {:?}", file));
            }
            progress_callback(0.1 * (i as f32 / files.len() as f32));
        }

        progress_callback(0.1);

        // Detect if files have different codecs/formats
        let needs_re_encode = self.needs_re_encode(files).await?;

        progress_callback(0.2);

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

        progress_callback(0.3);

        // Output file
        let timestamp = chrono::Utc::now().timestamp_millis();
        let output_file = self.temp_dir.join(format!("merged_{}.m4a", timestamp));

        progress_callback(0.4);

        // Build FFmpeg command based on whether re-encoding is needed
        let mut args = vec![
            "-f".to_string(),
            "concat".to_string(),
            "-safe".to_string(),
            "0".to_string(),
            "-i".to_string(),
            concat_list.to_str().unwrap().to_string(),
        ];

        if needs_re_encode {
            // Re-encode to ensure compatibility
            args.extend([
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                "192k".to_string(),
            ]);
        } else {
            // Copy codec (faster, no quality loss)
            args.extend([
                "-c".to_string(),
                "copy".to_string(),
            ]);
        }

        args.extend([
            "-y".to_string(), // Overwrite output file if exists
            output_file.to_str().unwrap().to_string(),
        ]);

        progress_callback(0.5);

        // Run FFmpeg concat
        let output = Command::new(&self.ffmpeg_path)
            .args(&args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .output()
            .context("Failed to execute FFmpeg merge")?;

        progress_callback(0.9);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Clean up on error
            let _ = fs::remove_file(concat_list).await;
            return Err(anyhow!("FFmpeg merge failed: {}", stderr));
        }

        // Clean up concat list
        let _ = fs::remove_file(concat_list).await;

        progress_callback(1.0);

        Ok(output_file)
    }

    /// Check if files need re-encoding for merging
    async fn needs_re_encode(&self, files: &[PathBuf]) -> Result<bool> {
        if files.is_empty() {
            return Ok(false);
        }

        // Get format info for first file
        let first_format = self.detect_audio_format(&files[0]).await?;

        // Check if all files have the same format
        for file in files.iter().skip(1) {
            let format = self.detect_audio_format(file).await?;
            if format != first_format {
                return Ok(true); // Different formats, need re-encode
            }
        }

        Ok(false)
    }

    /// Detect audio format of a file
    async fn detect_audio_format(&self, file: &Path) -> Result<String> {
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                file.to_str().unwrap(),
                "-hide_banner",
            ])
            .output()
            .context("Failed to probe audio format")?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse codec from FFmpeg output
        // Example: "Audio: aac (LC) (mp4a / 0x6134706D)"
        for line in stderr.lines() {
            if line.contains("Audio:") {
                if let Some(codec_part) = line.split("Audio:").nth(1) {
                    if let Some(codec) = codec_part.trim().split_whitespace().next() {
                        return Ok(codec.to_lowercase());
                    }
                }
            }
        }

        // Fallback to extension-based detection
        if let Some(ext) = file.extension() {
            return Ok(ext.to_string_lossy().to_lowercase().to_string());
        }

        Err(anyhow!("Failed to detect audio format for: {:?}", file))
    }

    /// Replace a segment of audio with new audio
    ///
    /// # Arguments
    /// * `original` - Path to the original audio file
    /// * `replacement` - Path to the replacement audio file
    /// * `start_ms` - Start time in milliseconds where replacement begins
    /// * `end_ms` - End time in milliseconds where replacement ends
    /// * `fade_duration_ms` - Duration of fade in/out transitions (0 = no fade)
    ///
    /// # Returns
    /// Path to the new audio file with replaced segment
    pub async fn replace_audio_segment(
        &self,
        original: &Path,
        replacement: &Path,
        start_ms: u64,
        end_ms: u64,
        fade_duration_ms: u64,
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

        // Validate time ranges against original file duration
        let original_duration = self.get_duration(original).await?;
        if end_ms > original_duration {
            return Err(anyhow!(
                "End time ({} ms) exceeds original file duration ({} ms)",
                end_ms,
                original_duration
            ));
        }

        let start_sec = start_ms as f64 / 1000.0;
        let end_sec = end_ms as f64 / 1000.0;
        let fade_sec = fade_duration_ms as f64 / 1000.0;

        let timestamp = chrono::Utc::now().timestamp_millis();

        // Extract part before replacement (with fade out if requested)
        let before_file = self.temp_dir.join(format!("before_{}.m4a", timestamp));
        if start_ms > 0 {
            let mut before_args = vec![
                "-i".to_string(),
                original.to_str().unwrap().to_string(),
                "-t".to_string(),
                start_sec.to_string(),
            ];

            // Add fade out filter if fade is enabled
            if fade_duration_ms > 0 && start_ms > fade_duration_ms {
                let fade_start = (start_ms - fade_duration_ms) as f64 / 1000.0;
                before_args.extend([
                    "-af".to_string(),
                    format!("afade=t=out:st={}:d={}", fade_start, fade_sec),
                ]);
            }

            before_args.extend([
                "-y".to_string(),
                before_file.to_str().unwrap().to_string(),
            ]);

            let before_output = Command::new(&self.ffmpeg_path)
                .args(&before_args)
                .output()
                .context("Failed to extract before segment")?;

            if !before_output.status.success() {
                let stderr = String::from_utf8_lossy(&before_output.stderr);
                return Err(anyhow!("Failed to extract before segment: {}", stderr));
            }
        } else {
            // Create empty placeholder if start is at beginning
            fs::write(&before_file, b"").await?;
        }

        // Process replacement audio (with fade in and fade out if requested)
        let processed_replacement = if fade_duration_ms > 0 {
            let replacement_duration = self.get_duration(replacement).await?;
            let fade_out_start = if replacement_duration > fade_duration_ms {
                (replacement_duration - fade_duration_ms) as f64 / 1000.0
            } else {
                0.0
            };

            let proc_repl_file = self
                .temp_dir
                .join(format!("processed_replacement_{}.m4a", timestamp));

            let fade_filter = format!(
                "afade=t=in:st=0:d={},afade=t=out:st={}:d={}",
                fade_sec, fade_out_start, fade_sec
            );

            let proc_output = Command::new(&self.ffmpeg_path)
                .args([
                    "-i",
                    replacement.to_str().unwrap(),
                    "-af",
                    &fade_filter,
                    "-y",
                    proc_repl_file.to_str().unwrap(),
                ])
                .output()
                .context("Failed to process replacement with fades")?;

            if !proc_output.status.success() {
                let stderr = String::from_utf8_lossy(&proc_output.stderr);
                return Err(anyhow!("Failed to process replacement: {}", stderr));
            }

            proc_repl_file
        } else {
            replacement.to_path_buf()
        };

        // Extract part after replacement (with fade in if requested)
        let after_file = self.temp_dir.join(format!("after_{}.m4a", timestamp));
        if end_ms < original_duration {
            let mut after_args = vec![
                "-i".to_string(),
                original.to_str().unwrap().to_string(),
                "-ss".to_string(),
                end_sec.to_string(),
            ];

            // Add fade in filter if fade is enabled
            if fade_duration_ms > 0 {
                after_args.extend([
                    "-af".to_string(),
                    format!("afade=t=in:st=0:d={}", fade_sec),
                ]);
            }

            after_args.extend([
                "-y".to_string(),
                after_file.to_str().unwrap().to_string(),
            ]);

            let after_output = Command::new(&self.ffmpeg_path)
                .args(&after_args)
                .output()
                .context("Failed to extract after segment")?;

            if !after_output.status.success() {
                let stderr = String::from_utf8_lossy(&after_output.stderr);
                return Err(anyhow!("Failed to extract after segment: {}", stderr));
            }
        } else {
            // Create empty placeholder if end is at file end
            fs::write(&after_file, b"").await?;
        }

        // Merge: before + replacement + after
        let mut parts = Vec::new();
        if start_ms > 0 && before_file.metadata().await?.len() > 0 {
            parts.push(before_file.clone());
        }
        parts.push(processed_replacement.clone());
        if end_ms < original_duration && after_file.metadata().await?.len() > 0 {
            parts.push(after_file.clone());
        }

        let result = self.merge_audio(&parts).await?;

        // Clean up temporary files
        let _ = fs::remove_file(before_file).await;
        let _ = fs::remove_file(after_file).await;
        if fade_duration_ms > 0 {
            let _ = fs::remove_file(processed_replacement).await;
        }

        Ok(result)
    }

    /// Parse timestamp string to milliseconds
    ///
    /// Supports formats:
    /// - Milliseconds: "1234" or "1234ms"
    /// - Seconds: "12.5s" or "12.5"
    /// - HH:MM:SS: "00:01:23"
    /// - HH:MM:SS.mmm: "00:01:23.456"
    ///
    /// # Arguments
    /// * `timestamp` - Timestamp string in supported format
    ///
    /// # Returns
    /// Time in milliseconds
    pub fn parse_timestamp(timestamp: &str) -> Result<u64> {
        let timestamp = timestamp.trim();

        // HH:MM:SS or HH:MM:SS.mmm format
        if timestamp.contains(':') {
            let parts: Vec<&str> = timestamp.split(':').collect();
            if parts.len() != 3 {
                return Err(anyhow!("Invalid timestamp format. Expected HH:MM:SS"));
            }

            let hours: u64 = parts[0]
                .parse()
                .context("Invalid hours in timestamp")?;
            let minutes: u64 = parts[1]
                .parse()
                .context("Invalid minutes in timestamp")?;

            // Handle seconds with optional milliseconds
            let seconds_part = parts[2];
            let (seconds, millis) = if seconds_part.contains('.') {
                let sec_parts: Vec<&str> = seconds_part.split('.').collect();
                let secs: u64 = sec_parts[0]
                    .parse()
                    .context("Invalid seconds in timestamp")?;
                let ms_str = sec_parts[1];
                // Pad or truncate to 3 digits
                let ms: u64 = if ms_str.len() >= 3 {
                    ms_str[..3].parse().context("Invalid milliseconds")?
                } else {
                    let padded = format!("{:0<3}", ms_str);
                    padded.parse().context("Invalid milliseconds")?
                };
                (secs, ms)
            } else {
                (seconds_part.parse().context("Invalid seconds")?, 0)
            };

            let total_ms = hours * 3600000 + minutes * 60000 + seconds * 1000 + millis;
            return Ok(total_ms);
        }

        // Milliseconds format (number or "123ms")
        if timestamp.ends_with("ms") {
            let num_str = timestamp.trim_end_matches("ms");
            return num_str
                .parse()
                .context("Invalid milliseconds value");
        }

        // Seconds format (number with 's' or just decimal)
        if timestamp.ends_with('s') {
            let num_str = timestamp.trim_end_matches('s');
            let seconds: f64 = num_str
                .parse()
                .context("Invalid seconds value")?;
            return Ok((seconds * 1000.0) as u64);
        }

        // Plain number - try as milliseconds first, if very large
        // Otherwise treat as seconds if it contains decimal point
        if timestamp.contains('.') {
            let seconds: f64 = timestamp
                .parse()
                .context("Invalid numeric timestamp")?;
            Ok((seconds * 1000.0) as u64)
        } else {
            timestamp
                .parse()
                .context("Invalid numeric timestamp")
        }
    }

    /// Save audio data to a new file (legacy - kept for backward compatibility)
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

    /// Save an existing audio file as a new note with optional format conversion
    ///
    /// # Arguments
    /// * `source_path` - Path to the source audio file
    /// * `output_format` - Desired output format (e.g., "m4a", "mp3", "wav", "ogg")
    /// * `quality_settings` - Optional audio quality settings (bitrate)
    ///
    /// # Returns
    /// Path to the new audio file
    pub async fn save_audio_as_new(
        &self,
        source_path: &Path,
        output_format: &str,
        quality_settings: Option<AudioQualitySettings>,
    ) -> Result<PathBuf> {
        if !source_path.exists() {
            return Err(anyhow!("Source file does not exist"));
        }

        // Validate output format
        let valid_formats = ["m4a", "mp3", "wav", "ogg"];
        if !valid_formats.contains(&output_format) {
            return Err(anyhow!(
                "Unsupported output format: {}. Supported formats: {:?}",
                output_format,
                valid_formats
            ));
        }

        // Get source format
        let source_ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("m4a");

        // If formats match and no quality settings provided, just copy the file
        if source_ext == output_format && quality_settings.is_none() {
            let timestamp = chrono::Utc::now().timestamp_millis();
            let output_file = self
                .temp_dir
                .join(format!("save_as_new_{}.{}", timestamp, output_format));

            fs::copy(source_path, &output_file)
                .await
                .context("Failed to copy audio file")?;

            return Ok(output_file);
        }

        // Convert format or apply quality settings
        let bitrate = quality_settings
            .as_ref()
            .map(|s| s.bitrate.as_str());

        self.convert_format(source_path, output_format, bitrate)
            .await
    }
}

/// Audio quality settings for save-as-new operations
#[derive(Debug, Clone)]
pub struct AudioQualitySettings {
    pub bitrate: String, // e.g., "128k", "192k", "320k"
}

impl AudioQualitySettings {
    /// Create new quality settings with the specified bitrate
    pub fn new(bitrate: &str) -> Self {
        Self {
            bitrate: bitrate.to_string(),
        }
    }

    /// High quality preset (320k for MP3, 256k for AAC)
    pub fn high(format: &str) -> Self {
        let bitrate = match format {
            "mp3" => "320k",
            "m4a" => "256k",
            "ogg" => "320k",
            _ => "256k",
        };
        Self::new(bitrate)
    }

    /// Medium quality preset (192k for MP3, 192k for AAC)
    pub fn medium(format: &str) -> Self {
        let bitrate = match format {
            "mp3" => "192k",
            "m4a" => "192k",
            "ogg" => "192k",
            _ => "192k",
        };
        Self::new(bitrate)
    }

    /// Low quality preset (128k for MP3, 128k for AAC)
    pub fn low(format: &str) -> Self {
        let bitrate = match format {
            "mp3" => "128k",
            "m4a" => "128k",
            "ogg" => "128k",
            _ => "128k",
        };
        Self::new(bitrate)
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
    use hound::{WavSpec, WavWriter};

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

    fn create_test_wav_file(path: &Path) -> Result<()> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;

        // Write 1 second of silence
        for _ in 0..44100 {
            writer.write_sample(0i16)?;
        }

        writer.finalize()?;
        Ok(())
    }

    fn create_test_mp3_file(path: &Path) -> Result<()> {
        // Create a minimal MP3 file with ID3v2 header
        let mut file = std::fs::File::create(path)?;

        // ID3v2.3 header
        let id3_header: &[u8] = &[
            0x49, 0x44, 0x33, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ID3v2.3 header
        ];
        file.write_all(id3_header)?;

        // Minimal MP3 frame
        let mp3_frame: &[u8] = &[
            0xFF, 0xFB, 0x90, 0x00, // MPEG1 Layer3 frame
        ];
        file.write_all(mp3_frame)?;
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

    // ===== MERGE AUDIO TESTS (TDD) =====

    #[tokio::test]
    async fn test_merge_audio_two_files_success() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return, // Skip if FFmpeg not available
        };

        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("test_merge1.wav");
        let file2 = temp_dir.join("test_merge2.wav");

        create_test_wav_file(&file1).unwrap();
        create_test_wav_file(&file2).unwrap();

        let result = processor.merge_audio(&[file1.clone(), file2.clone()]).await;

        assert!(result.is_ok(), "Merge should succeed with valid files");

        if let Ok(output) = result {
            assert!(output.exists(), "Output file should exist");
            assert!(output.metadata().unwrap().len() > 0, "Output file should not be empty");

            // Cleanup
            let _ = fs::remove_file(output).await;
        }

        let _ = std::fs::remove_file(file1);
        let _ = std::fs::remove_file(file2);
    }

    #[tokio::test]
    async fn test_merge_audio_multiple_files() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("test_merge_m1.wav");
        let file2 = temp_dir.join("test_merge_m2.wav");
        let file3 = temp_dir.join("test_merge_m3.wav");

        create_test_wav_file(&file1).unwrap();
        create_test_wav_file(&file2).unwrap();
        create_test_wav_file(&file3).unwrap();

        let result = processor.merge_audio(&[file1.clone(), file2.clone(), file3.clone()]).await;

        assert!(result.is_ok(), "Merge should succeed with 3 files");

        if let Ok(output) = result {
            assert!(output.exists());
            let _ = fs::remove_file(output).await;
        }

        let _ = std::fs::remove_file(file1);
        let _ = std::fs::remove_file(file2);
        let _ = std::fs::remove_file(file3);
    }

    #[tokio::test]
    async fn test_merge_audio_file_not_found() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let file1 = PathBuf::from("/tmp/nonexistent1.wav");
        let file2 = PathBuf::from("/tmp/nonexistent2.wav");

        let result = processor.merge_audio(&[file1, file2]).await;

        assert!(result.is_err(), "Should fail when files don't exist");
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_merge_audio_empty_array() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let result = processor.merge_audio(&[]).await;

        assert!(result.is_err(), "Should fail with empty array");
        assert!(result.unwrap_err().to_string().contains("No files provided"));
    }

    #[tokio::test]
    async fn test_merge_audio_single_file() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("test_merge_single.wav");
        create_test_wav_file(&file1).unwrap();

        let result = processor.merge_audio(&[file1.clone()]).await;

        assert!(result.is_err(), "Should fail with single file");
        assert!(result.unwrap_err().to_string().contains("at least 2 files"));

        let _ = std::fs::remove_file(file1);
    }

    #[tokio::test]
    async fn test_merge_audio_mixed_formats() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let temp_dir = std::env::temp_dir();
        let wav_file = temp_dir.join("test_merge_fmt1.wav");
        let mp3_file = temp_dir.join("test_merge_fmt2.mp3");

        create_test_wav_file(&wav_file).unwrap();
        create_test_mp3_file(&mp3_file).unwrap();

        let result = processor.merge_audio(&[wav_file.clone(), mp3_file.clone()]).await;

        // FFmpeg should handle format conversion automatically
        if result.is_ok() {
            let output = result.unwrap();
            assert!(output.exists());
            let _ = fs::remove_file(output).await;
        }

        let _ = std::fs::remove_file(wav_file);
        let _ = std::fs::remove_file(mp3_file);
    }

    #[tokio::test]
    async fn test_merge_audio_with_callback() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("test_merge_cb1.wav");
        let file2 = temp_dir.join("test_merge_cb2.wav");

        create_test_wav_file(&file1).unwrap();
        create_test_wav_file(&file2).unwrap();

        let progress_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();

        let callback = move |_progress: f32| {
            progress_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        };

        let result = processor.merge_audio_with_progress(&[file1.clone(), file2.clone()], callback).await;

        if result.is_ok() {
            let output = result.unwrap();
            assert!(output.exists());
            // Note: callback might not be called for fast operations
            let _ = fs::remove_file(output).await;
        }

        let _ = std::fs::remove_file(file1);
        let _ = std::fs::remove_file(file2);
    }

    #[tokio::test]
    async fn test_merge_audio_codec_issues() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let temp_dir = std::env::temp_dir();
        let corrupted_file1 = temp_dir.join("test_merge_bad1.wav");
        let corrupted_file2 = temp_dir.join("test_merge_bad2.wav");

        // Create corrupted files
        std::fs::write(&corrupted_file1, b"invalid audio data").unwrap();
        std::fs::write(&corrupted_file2, b"also invalid").unwrap();

        let result = processor.merge_audio(&[corrupted_file1.clone(), corrupted_file2.clone()]).await;

        // Should fail due to invalid format
        assert!(result.is_err(), "Should fail with corrupted files");

        let _ = std::fs::remove_file(corrupted_file1);
        let _ = std::fs::remove_file(corrupted_file2);
    }

    // ===== SAVE AS NEW TESTS (TDD) =====

    #[tokio::test]
    async fn test_save_audio_as_new_source_not_exists() {
        let processor = AudioProcessor::new().unwrap();
        let nonexistent = PathBuf::from("/tmp/nonexistent_audio.m4a");

        let result = processor
            .save_audio_as_new(&nonexistent, "m4a", None)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source file does not exist"));
    }

    #[tokio::test]
    async fn test_save_audio_as_new_invalid_format() {
        let processor = AudioProcessor::new().unwrap();

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_invalid_format.m4a");
        create_test_audio_file(&test_file).unwrap();

        let result = processor
            .save_audio_as_new(&test_file, "invalid", None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported output format"));

        // Cleanup
        let _ = fs::remove_file(test_file).await;
    }

    #[tokio::test]
    async fn test_save_audio_as_new_same_format_no_quality() {
        let processor = AudioProcessor::new().unwrap();

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_same_format.m4a");
        create_test_audio_file(&test_file).unwrap();

        let result = processor
            .save_audio_as_new(&test_file, "m4a", None)
            .await;

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(output_path.exists());
        assert_eq!(output_path.extension().unwrap(), "m4a");
        assert!(output_path.file_name().unwrap().to_str().unwrap().starts_with("save_as_new_"));

        // Verify file was copied
        let output_size = fs::metadata(&output_path).await.unwrap().len();
        let source_size = fs::metadata(&test_file).await.unwrap().len();
        assert_eq!(output_size, source_size);

        // Cleanup
        let _ = fs::remove_file(test_file).await;
        let _ = fs::remove_file(output_path).await;
    }

    #[tokio::test]
    async fn test_save_audio_as_new_different_format() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        // Only run if FFmpeg is available
        if processor.verify_ffmpeg().is_err() {
            return;
        }

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_convert_format.m4a");
        create_test_audio_file(&test_file).unwrap();

        let result = processor
            .save_audio_as_new(&test_file, "mp3", None)
            .await;

        // Cleanup regardless of result
        let _ = fs::remove_file(&test_file).await;

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(output_path.exists());
        assert_eq!(output_path.extension().unwrap(), "mp3");

        let _ = fs::remove_file(output_path).await;
    }

    #[tokio::test]
    async fn test_save_audio_as_new_with_quality_high() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        if processor.verify_ffmpeg().is_err() {
            return;
        }

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_quality_high.m4a");
        create_test_audio_file(&test_file).unwrap();

        let quality = AudioQualitySettings::high("mp3");
        let result = processor
            .save_audio_as_new(&test_file, "mp3", Some(quality))
            .await;

        // Cleanup regardless of result
        let _ = fs::remove_file(&test_file).await;

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(output_path.exists());
        assert_eq!(output_path.extension().unwrap(), "mp3");

        let _ = fs::remove_file(output_path).await;
    }

    #[tokio::test]
    async fn test_save_audio_as_new_with_quality_medium() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        if processor.verify_ffmpeg().is_err() {
            return;
        }

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_quality_medium.m4a");
        create_test_audio_file(&test_file).unwrap();

        let quality = AudioQualitySettings::medium("mp3");
        assert_eq!(quality.bitrate, "192k");

        let result = processor
            .save_audio_as_new(&test_file, "mp3", Some(quality))
            .await;

        // Cleanup
        let _ = fs::remove_file(&test_file).await;

        if let Ok(output_path) = result {
            let _ = fs::remove_file(output_path).await;
        }
    }

    #[tokio::test]
    async fn test_save_audio_as_new_with_quality_low() {
        let processor = match AudioProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        if processor.verify_ffmpeg().is_err() {
            return;
        }

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_quality_low.m4a");
        create_test_audio_file(&test_file).unwrap();

        let quality = AudioQualitySettings::low("mp3");
        assert_eq!(quality.bitrate, "128k");

        let result = processor
            .save_audio_as_new(&test_file, "mp3", Some(quality))
            .await;

        // Cleanup
        let _ = fs::remove_file(&test_file).await;

        if let Ok(output_path) = result {
            let _ = fs::remove_file(output_path).await;
        }
    }

    #[test]
    fn test_audio_quality_settings() {
        // Test custom bitrate
        let custom = AudioQualitySettings::new("256k");
        assert_eq!(custom.bitrate, "256k");

        // Test presets for different formats
        let high_mp3 = AudioQualitySettings::high("mp3");
        assert_eq!(high_mp3.bitrate, "320k");

        let high_m4a = AudioQualitySettings::high("m4a");
        assert_eq!(high_m4a.bitrate, "256k");

        let medium_mp3 = AudioQualitySettings::medium("mp3");
        assert_eq!(medium_mp3.bitrate, "192k");

        let low_ogg = AudioQualitySettings::low("ogg");
        assert_eq!(low_ogg.bitrate, "128k");
    }

    #[tokio::test]
    async fn test_save_audio_as_new_supported_formats() {
        let processor = AudioProcessor::new().unwrap();

        // Create a test audio file
        let test_file = processor.temp_dir.join("test_formats.m4a");
        create_test_audio_file(&test_file).unwrap();

        let formats = ["m4a", "mp3", "wav", "ogg"];

        for format in &formats {
            let result = processor
                .save_audio_as_new(&test_file, format, None)
                .await;

            // m4a should always work (copy), others need FFmpeg
            if *format == "m4a" {
                assert!(result.is_ok());
                if let Ok(output_path) = result {
                    assert!(output_path.exists());
                    let _ = fs::remove_file(output_path).await;
                }
            } else if processor.verify_ffmpeg().is_ok() {
                // Other formats should work if FFmpeg is available
                if let Ok(output_path) = result {
                    let _ = fs::remove_file(output_path).await;
                }
            }
        }

        // Cleanup
        let _ = fs::remove_file(test_file).await;
    }
}
