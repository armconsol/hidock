use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("FFmpeg binary not found at path: {0}")]
    BinaryNotFound(String),

    #[error("FFmpeg execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid FFmpeg output: {0}")]
    InvalidOutput(String),

    #[error("FFmpeg version validation failed: {0}")]
    VersionValidationFailed(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),
}

/// FFmpeg wrapper for audio processing operations
pub struct FFmpegWrapper {
    binary_path: PathBuf,
}

impl FFmpegWrapper {
    /// Create a new FFmpegWrapper with automatic binary detection
    pub fn new() -> Result<Self> {
        let binary_path = Self::detect_ffmpeg_binary()?;
        Ok(Self { binary_path })
    }

    /// Create a new FFmpegWrapper with a specific binary path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let binary_path = path.as_ref().to_path_buf();

        if !binary_path.exists() {
            return Err(FFmpegError::BinaryNotFound(binary_path.display().to_string()).into());
        }

        Ok(Self { binary_path })
    }

    /// Detect FFmpeg binary based on platform and common locations
    fn detect_ffmpeg_binary() -> Result<PathBuf> {
        let platform = std::env::consts::OS;

        // Try common system paths first
        let system_paths = match platform {
            "macos" => vec![
                "/usr/local/bin/ffmpeg",
                "/opt/homebrew/bin/ffmpeg",
                "/usr/bin/ffmpeg",
            ],
            "linux" => vec![
                "/usr/bin/ffmpeg",
                "/usr/local/bin/ffmpeg",
                "/snap/bin/ffmpeg",
            ],
            "windows" => vec![
                "C:\\Program Files\\ffmpeg\\bin\\ffmpeg.exe",
                "C:\\ffmpeg\\bin\\ffmpeg.exe",
            ],
            _ => return Err(FFmpegError::UnsupportedPlatform(platform.to_string()).into()),
        };

        // Check system paths
        for path in &system_paths {
            let pb = PathBuf::from(path);
            if pb.exists() {
                return Ok(pb);
            }
        }

        // Try bundled binary in Tauri resources
        if let Ok(resource_path) = Self::get_bundled_ffmpeg_path() {
            if resource_path.exists() {
                return Ok(resource_path);
            }
        }

        // Try PATH environment variable
        if let Ok(output) = Command::new("which").arg("ffmpeg").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let pb = PathBuf::from(path_str);
                if pb.exists() {
                    return Ok(pb);
                }
            }
        }

        Err(
            FFmpegError::BinaryNotFound("FFmpeg not found in any standard location".to_string())
                .into(),
        )
    }

    /// Get path to bundled FFmpeg binary in Tauri resources
    fn get_bundled_ffmpeg_path() -> Result<PathBuf> {
        let platform = std::env::consts::OS;
        let binary_name = match platform {
            "windows" => "ffmpeg.exe",
            _ => "ffmpeg",
        };

        // In development, look in src-tauri/binaries/
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(platform)
            .join(binary_name);

        if dev_path.exists() {
            return Ok(dev_path);
        }

        // In production, Tauri bundles external binaries in a platform-specific location
        // This is a placeholder - actual implementation depends on Tauri resource handling
        Err(anyhow!("Bundled FFmpeg not found"))
    }

    /// Validate that FFmpeg is working and return version information
    pub fn validate(&self) -> Result<String> {
        let output = self.execute_command(&["-version"])?;
        let version_output = String::from_utf8_lossy(&output.stdout);

        // Parse first line which contains version info
        let version = version_output
            .lines()
            .next()
            .ok_or_else(|| FFmpegError::VersionValidationFailed("Empty output".to_string()))?
            .to_string();

        if !version.starts_with("ffmpeg version") {
            return Err(FFmpegError::VersionValidationFailed(format!(
                "Unexpected version format: {}",
                version
            ))
            .into());
        }

        Ok(version)
    }

    /// Execute FFmpeg command with given arguments
    fn execute_command(&self, args: &[&str]) -> Result<Output> {
        let output = Command::new(&self.binary_path)
            .args(args)
            .output()
            .context("Failed to execute FFmpeg command")?;

        Ok(output)
    }

    /// Convert audio file to a different format
    pub fn convert_audio<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        output_format: &str,
    ) -> Result<()> {
        let input = input_path.as_ref();
        let output = output_path.as_ref();

        if !input.exists() {
            return Err(anyhow!("Input file does not exist: {}", input.display()));
        }

        let args = vec![
            "-i",
            input
                .to_str()
                .ok_or_else(|| anyhow!("Invalid input path"))?,
            "-y", // Overwrite output file if exists
            "-f",
            output_format,
            output
                .to_str()
                .ok_or_else(|| anyhow!("Invalid output path"))?,
        ];

        let output = self.execute_command(&args)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FFmpegError::ExecutionFailed(format!(
                "FFmpeg conversion failed: {}",
                stderr
            ))
            .into());
        }

        Ok(())
    }

    /// Merge multiple audio files into a single file
    pub fn merge_audio_files<P: AsRef<Path>>(
        &self,
        input_paths: &[P],
        output_path: P,
        output_format: &str,
    ) -> Result<()> {
        if input_paths.is_empty() {
            return Err(anyhow!("No input files provided"));
        }

        let output = output_path.as_ref();

        // Verify all input files exist
        for input in input_paths {
            if !input.as_ref().exists() {
                return Err(anyhow!(
                    "Input file does not exist: {}",
                    input.as_ref().display()
                ));
            }
        }

        // Build FFmpeg concat filter
        let mut args = vec!["-y"];

        // Add all input files
        for input in input_paths {
            args.push("-i");
            args.push(
                input
                    .as_ref()
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid input path"))?,
            );
        }

        // Create filter complex for concatenation
        let filter_inputs = (0..input_paths.len())
            .map(|i| format!("[{}:a]", i))
            .collect::<Vec<_>>()
            .join("");
        let filter_complex = format!(
            "{}concat=n={}:v=0:a=1[out]",
            filter_inputs,
            input_paths.len()
        );

        args.extend_from_slice(&[
            "-filter_complex",
            &filter_complex,
            "-map",
            "[out]",
            "-f",
            output_format,
            output
                .to_str()
                .ok_or_else(|| anyhow!("Invalid output path"))?,
        ]);

        let output = self.execute_command(&args)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(
                FFmpegError::ExecutionFailed(format!("FFmpeg merge failed: {}", stderr)).into(),
            );
        }

        Ok(())
    }

    /// Extract audio segment from a file
    pub fn extract_segment<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        start_time: f64,
        duration: f64,
        output_format: &str,
    ) -> Result<()> {
        let input = input_path.as_ref();
        let output = output_path.as_ref();

        if !input.exists() {
            return Err(anyhow!("Input file does not exist: {}", input.display()));
        }

        if start_time < 0.0 || duration <= 0.0 {
            return Err(anyhow!(
                "Invalid time parameters: start={}, duration={}",
                start_time,
                duration
            ));
        }

        let start_time_str = start_time.to_string();
        let duration_str = duration.to_string();

        let args = vec![
            "-i",
            input
                .to_str()
                .ok_or_else(|| anyhow!("Invalid input path"))?,
            "-ss",
            &start_time_str,
            "-t",
            &duration_str,
            "-y",
            "-f",
            output_format,
            output
                .to_str()
                .ok_or_else(|| anyhow!("Invalid output path"))?,
        ];

        let output = self.execute_command(&args)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(
                FFmpegError::ExecutionFailed(format!("FFmpeg extract failed: {}", stderr)).into(),
            );
        }

        Ok(())
    }

    /// Get audio file metadata (duration, format, bitrate, etc.)
    pub fn get_audio_info<P: AsRef<Path>>(&self, input_path: P) -> Result<AudioInfo> {
        let input = input_path.as_ref();

        if !input.exists() {
            return Err(anyhow!("Input file does not exist: {}", input.display()));
        }

        let args = vec![
            "-i",
            input
                .to_str()
                .ok_or_else(|| anyhow!("Invalid input path"))?,
            "-hide_banner",
        ];

        let output = self.execute_command(&args)?;
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse duration from output
        let duration = Self::parse_duration(&stderr)?;

        Ok(AudioInfo { duration })
    }

    /// Parse duration from FFmpeg output
    fn parse_duration(output: &str) -> Result<f64> {
        for line in output.lines() {
            if line.contains("Duration:") {
                // Example: "  Duration: 00:01:23.45, start: 0.000000, bitrate: 128 kb/s"
                if let Some(duration_str) = line.split("Duration:").nth(1) {
                    if let Some(time_str) = duration_str.split(',').next() {
                        let time_str = time_str.trim();
                        // Parse HH:MM:SS.ss format
                        let parts: Vec<&str> = time_str.split(':').collect();
                        if parts.len() == 3 {
                            let hours: f64 = parts[0].parse()?;
                            let minutes: f64 = parts[1].parse()?;
                            let seconds: f64 = parts[2].parse()?;
                            return Ok(hours * 3600.0 + minutes * 60.0 + seconds);
                        }
                    }
                }
            }
        }

        Err(anyhow!("Could not parse duration from FFmpeg output"))
    }

    /// Get the path to the FFmpeg binary
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }
}

/// Audio file metadata
#[derive(Debug, Clone)]
pub struct AudioInfo {
    pub duration: f64, // Duration in seconds
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ffmpeg_wrapper_new_should_fail_without_binary() {
        // This test will fail if FFmpeg is actually installed
        // We'll skip it in CI or when FFmpeg is available
        if Command::new("which").arg("ffmpeg").output().is_ok() {
            // FFmpeg exists, skip this test
            return;
        }

        let result = FFmpegWrapper::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_ffmpeg_wrapper_with_invalid_path_should_fail() {
        let result = FFmpegWrapper::with_path("/nonexistent/path/to/ffmpeg");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("not found"));
        }
    }

    #[test]
    fn test_ffmpeg_wrapper_with_valid_path_should_succeed() {
        // Try to find FFmpeg in system
        let ffmpeg_paths = vec![
            "/usr/local/bin/ffmpeg",
            "/opt/homebrew/bin/ffmpeg",
            "/usr/bin/ffmpeg",
        ];

        let mut found_path = None;
        for path in ffmpeg_paths {
            if PathBuf::from(path).exists() {
                found_path = Some(path);
                break;
            }
        }

        if let Some(path) = found_path {
            let result = FFmpegWrapper::with_path(path);
            assert!(result.is_ok());

            let wrapper = result.unwrap();
            assert_eq!(wrapper.binary_path(), Path::new(path));
        }
    }

    #[test]
    fn test_validate_should_return_version_info() {
        // Skip if FFmpeg not available
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let result = wrapper.validate();
        assert!(result.is_ok());

        let version = result.unwrap();
        assert!(version.starts_with("ffmpeg version"));
    }

    #[test]
    fn test_convert_audio_with_nonexistent_input_should_fail() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("nonexistent.wav");
        let output_path = temp_dir.path().join("output.mp3");

        let result = wrapper.convert_audio(&input_path, &output_path, "mp3");
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_audio_should_succeed_with_valid_input() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.wav");
        let output_path = temp_dir.path().join("output.mp3");

        // Create a minimal valid WAV file (1 second of silence)
        create_test_wav_file(&input_path).unwrap();

        let result = wrapper.convert_audio(&input_path, &output_path, "mp3");
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_merge_audio_files_with_empty_input_should_fail() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.mp3");

        let empty_vec: Vec<PathBuf> = vec![];
        let result = wrapper.merge_audio_files(&empty_vec, output_path, "mp3");
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_audio_files_should_succeed_with_valid_inputs() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let input1 = temp_dir.path().join("input1.wav");
        let input2 = temp_dir.path().join("input2.wav");
        let output_path = temp_dir.path().join("output.wav");

        create_test_wav_file(&input1).unwrap();
        create_test_wav_file(&input2).unwrap();

        let result = wrapper.merge_audio_files(&[&input1, &input2], &output_path, "wav");
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_extract_segment_with_invalid_times_should_fail() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.wav");
        let output_path = temp_dir.path().join("output.wav");

        create_test_wav_file(&input_path).unwrap();

        // Negative start time should fail
        let result = wrapper.extract_segment(&input_path, &output_path, -1.0, 1.0, "wav");
        assert!(result.is_err());

        // Zero duration should fail
        let result = wrapper.extract_segment(&input_path, &output_path, 0.0, 0.0, "wav");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_segment_should_succeed_with_valid_params() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.wav");
        let output_path = temp_dir.path().join("output.wav");

        create_test_wav_file(&input_path).unwrap();

        let result = wrapper.extract_segment(&input_path, &output_path, 0.0, 0.5, "wav");
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_get_audio_info_should_return_duration() {
        let wrapper = match FFmpegWrapper::new() {
            Ok(w) => w,
            Err(_) => return,
        };

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.wav");

        create_test_wav_file(&input_path).unwrap();

        let result = wrapper.get_audio_info(&input_path);
        assert!(result.is_ok());

        let info = result.unwrap();
        // The test file is approximately 1 second
        assert!(info.duration > 0.9 && info.duration < 1.1);
    }

    #[test]
    fn test_parse_duration_should_extract_correct_value() {
        let output = "  Duration: 00:01:23.45, start: 0.000000, bitrate: 128 kb/s";
        let duration = FFmpegWrapper::parse_duration(output).unwrap();
        assert!((duration - 83.45).abs() < 0.01);

        let output = "  Duration: 01:30:45.67, start: 0.000000, bitrate: 128 kb/s";
        let duration = FFmpegWrapper::parse_duration(output).unwrap();
        assert!((duration - 5445.67).abs() < 0.01);
    }

    /// Helper function to create a minimal valid WAV file for testing
    fn create_test_wav_file(path: &Path) -> Result<()> {
        use hound::{WavSpec, WavWriter};

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
}
