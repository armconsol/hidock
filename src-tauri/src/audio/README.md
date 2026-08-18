# Audio Module

This module provides audio processing capabilities for HiNotes Desktop.

## Components

### FFmpeg Integration (`ffmpeg.rs`)

The FFmpeg wrapper provides cross-platform audio processing:

- **Audio Conversion**: Convert between formats (WAV, MP3, AAC, OGG, etc.)
- **Audio Merging**: Concatenate multiple audio files
- **Segment Extraction**: Extract time-based segments from audio files
- **Metadata Extraction**: Get audio file information (duration, format, etc.)

**Usage Example:**

```rust
use hinotes_desktop_lib::audio::FFmpegWrapper;

// Initialize wrapper (auto-detects FFmpeg binary)
let wrapper = FFmpegWrapper::new()?;

// Validate installation
let version = wrapper.validate()?;
println!("FFmpeg version: {}", version);

// Convert audio
wrapper.convert_audio("input.wav", "output.mp3", "mp3")?;

// Get audio info
let info = wrapper.get_audio_info("audio.mp3")?;
println!("Duration: {} seconds", info.duration);
```

### Audio Cache (`cache.rs`)

Audio file caching system for improved performance.

### Audio Processor (`processor.rs`)

Core audio processing and transcription coordination.

### Diarization (`diarization.rs`)

Speaker diarization (speaker identification and segmentation) for multi-speaker audio.

## Platform Support

### FFmpeg Binary Detection

The FFmpeg wrapper automatically detects binaries in the following locations:

**macOS:**
- `/usr/local/bin/ffmpeg` (Intel Homebrew)
- `/opt/homebrew/bin/ffmpeg` (Apple Silicon Homebrew)
- `/usr/bin/ffmpeg` (System)

**Linux:**
- `/usr/bin/ffmpeg` (System)
- `/usr/local/bin/ffmpeg` (User install)
- `/snap/bin/ffmpeg` (Snap package)

**Windows:**
- `C:\Program Files\ffmpeg\bin\ffmpeg.exe`
- `C:\ffmpeg\bin\ffmpeg.exe`

Falls back to PATH environment variable if not found in standard locations.

## Testing

All audio modules follow Test-Driven Development (TDD) principles.

### Running Tests

```bash
# Run all audio module tests
cargo test --lib audio

# Run only FFmpeg tests
cargo test --lib audio::ffmpeg::tests

# Run example demo
cargo run --example ffmpeg_demo path/to/audio.wav
```

### Test Coverage

- ✅ Binary detection and validation
- ✅ Audio format conversion
- ✅ Multi-file merging
- ✅ Segment extraction with time boundaries
- ✅ Metadata parsing
- ✅ Error handling for invalid inputs

Tests automatically skip if FFmpeg is not installed.

## Installation

### Development Setup

Install FFmpeg via your system package manager:

**macOS:**
```bash
brew install ffmpeg
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install ffmpeg
```

**Linux (Fedora):**
```bash
sudo dnf install ffmpeg
```

**Windows:**
Download from:
- https://www.gyan.dev/ffmpeg/builds/
- https://github.com/BtbN/FFmpeg-Builds/releases

### Production Deployment

For bundled releases, place FFmpeg binaries in:
```
src-tauri/binaries/
├── macos/ffmpeg
├── linux/ffmpeg
└── windows/ffmpeg.exe
```

## Error Handling

The FFmpeg wrapper uses custom error types:

```rust
pub enum FFmpegError {
    BinaryNotFound(String),      // FFmpeg not installed
    ExecutionFailed(String),      // FFmpeg command failed
    InvalidOutput(String),        // Unexpected FFmpeg output
    VersionValidationFailed(String), // Version check failed
    UnsupportedPlatform(String),  // Platform not supported
}
```

All errors implement `std::error::Error` and can be converted to `anyhow::Error`.

## Performance Considerations

- FFmpeg operations are CPU-intensive and may block
- For UI responsiveness, wrap operations in async tasks
- Consider implementing progress callbacks for long operations
- Temporary files are not automatically cleaned up

## Security Notes

- Input/output paths are validated before passing to FFmpeg
- No shell execution (uses direct command invocation)
- Binary path is verified to exist before execution
- User-provided format strings are passed directly to FFmpeg

## Future Enhancements

- [ ] Progress callbacks with FFmpeg output parsing
- [ ] Cancellation support for long-running operations
- [ ] Audio normalization and noise reduction
- [ ] Batch processing with parallel execution
- [ ] Format detection without conversion
- [ ] Custom FFmpeg parameter support

## Documentation

For detailed information, see:
- [FFmpeg Integration Guide](../../FFmpeg_Integration.md)
- [FFmpeg Official Documentation](https://ffmpeg.org/documentation.html)
- [Tauri Commands Reference](../../src/commands/ffmpeg.rs)

## License

This module is part of HiNotes Desktop. See LICENSE file for details.
