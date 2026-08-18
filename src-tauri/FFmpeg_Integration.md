# FFmpeg Integration for HiNotes Desktop

## Overview

This document describes the FFmpeg integration implemented for HiNotes Desktop. FFmpeg is used for audio processing operations including format conversion, merging, and segment extraction.

## Files Created

### Core Implementation

- **src/audio/ffmpeg.rs** - Main FFmpeg wrapper with TDD tests
  - `FFmpegWrapper` struct for executing FFmpeg operations
  - Platform-specific binary detection (macOS, Linux, Windows)
  - Audio conversion, merging, and extraction functions
  - Comprehensive test suite

- **src/commands/ffmpeg.rs** - Tauri IPC commands for frontend integration
  - `FFmpegState` for managing FFmpeg instance
  - Async Tauri commands: `ffmpeg_validate`, `ffmpeg_convert_audio`, etc.
  - Error handling and state management

### Build Configuration

- **src-tauri/build.rs** - Build-time FFmpeg setup
  - Optional FFmpeg download (via `FFMPEG_DOWNLOAD=1` env var)
  - Platform detection and binary path management
  - Installation instructions for manual setup

- **src-tauri/tauri.conf.json** - Tauri bundling configuration
  - External binary bundling (commented out until binaries are available)
  - Cross-platform support

## Architecture

### Binary Detection Strategy

The FFmpegWrapper detects FFmpeg binaries in the following order:

1. **System Paths** (platform-specific)
   - macOS: `/usr/local/bin/ffmpeg`, `/opt/homebrew/bin/ffmpeg`, `/usr/bin/ffmpeg`
   - Linux: `/usr/bin/ffmpeg`, `/usr/local/bin/ffmpeg`, `/snap/bin/ffmpeg`
   - Windows: `C:\Program Files\ffmpeg\bin\ffmpeg.exe`, `C:\ffmpeg\bin\ffmpeg.exe`

2. **Bundled Binary** (development)
   - Location: `src-tauri/binaries/{platform}/ffmpeg[.exe]`

3. **PATH Environment Variable**
   - Uses `which ffmpeg` to locate system-installed FFmpeg

### Supported Operations

#### 1. Audio Conversion
```rust
wrapper.convert_audio(input_path, output_path, "mp3")?;
```
Converts audio files between formats (WAV, MP3, AAC, OGG, etc.)

#### 2. Audio Merging
```rust
wrapper.merge_audio_files(&[input1, input2], output_path, "wav")?;
```
Concatenates multiple audio files into a single file

#### 3. Segment Extraction
```rust
wrapper.extract_segment(input_path, output_path, start_time, duration, "mp3")?;
```
Extracts a time-based segment from an audio file

#### 4. Audio Metadata
```rust
let info = wrapper.get_audio_info(input_path)?;
println!("Duration: {} seconds", info.duration);
```
Retrieves audio file metadata (duration, format, etc.)

## Installation

### Development Setup

#### Option 1: System Installation (Recommended)

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

Extract and add to PATH.

#### Option 2: Bundled Binary (Future)

Set environment variable during build:
```bash
FFMPEG_DOWNLOAD=1 cargo build
```

Note: Automatic download is not yet implemented. Manual installation required.

### Production Deployment

For production builds, FFmpeg binaries should be:
1. Downloaded from official sources
2. Placed in `src-tauri/binaries/{platform}/`
3. Bundled with the app via `tauri.conf.json` (currently disabled)

## Testing

### Running Tests

```bash
cd src-tauri
cargo test --lib audio::ffmpeg::tests
```

### Test Coverage

The test suite includes:
- ✅ Binary detection and path validation
- ✅ Version validation
- ✅ Audio conversion with valid/invalid inputs
- ✅ Audio merging with multiple files
- ✅ Segment extraction with time validation
- ✅ Audio metadata parsing
- ✅ Duration parsing from FFmpeg output
- ✅ Error handling for missing files and invalid parameters

Tests automatically skip if FFmpeg is not installed on the system.

### Test Approach (TDD)

All functionality was implemented using Test-Driven Development:
1. Write failing tests first
2. Implement minimal code to pass tests
3. Refactor while keeping tests green

## Frontend Integration

### TypeScript/JavaScript Usage

```typescript
import { invoke } from '@tauri-apps/api/core';

// Validate FFmpeg
const version = await invoke('ffmpeg_validate');
console.log('FFmpeg version:', version);

// Convert audio
await invoke('ffmpeg_convert_audio', {
  inputPath: '/path/to/input.wav',
  outputPath: '/path/to/output.mp3',
  outputFormat: 'mp3'
});

// Merge audio files
await invoke('ffmpeg_merge_audio', {
  inputPaths: ['/path/to/file1.wav', '/path/to/file2.wav'],
  outputPath: '/path/to/merged.wav',
  outputFormat: 'wav'
});

// Extract segment
await invoke('ffmpeg_extract_segment', {
  inputPath: '/path/to/input.mp3',
  outputPath: '/path/to/segment.mp3',
  startTime: 10.5,
  duration: 30.0,
  outputFormat: 'mp3'
});

// Get audio info
const info = await invoke('ffmpeg_get_audio_info', {
  inputPath: '/path/to/audio.mp3'
});
console.log('Duration:', info.duration, 'seconds');
```

## Error Handling

### Error Types

```rust
pub enum FFmpegError {
    BinaryNotFound(String),
    ExecutionFailed(String),
    InvalidOutput(String),
    VersionValidationFailed(String),
    UnsupportedPlatform(String),
}
```

All errors are converted to strings for Tauri IPC compatibility.

### Common Error Scenarios

1. **FFmpeg not installed**
   - Error: "FFmpeg not found in any standard location"
   - Solution: Install FFmpeg via system package manager

2. **Invalid input file**
   - Error: "Input file does not exist"
   - Solution: Verify file path before calling FFmpeg

3. **Conversion failed**
   - Error: "FFmpeg conversion failed: [stderr output]"
   - Solution: Check FFmpeg stderr for specific error details

4. **Platform not supported**
   - Error: "Platform not supported: [platform]"
   - Solution: Add platform-specific paths to binary detection

## Implementation Notes

### Platform-Specific Considerations

**macOS:**
- Homebrew installs to `/opt/homebrew/bin` on Apple Silicon
- Intel Macs use `/usr/local/bin`
- Both paths are checked automatically

**Linux:**
- Snap packages install to `/snap/bin`
- Standard package managers use `/usr/bin` or `/usr/local/bin`
- All common locations are checked

**Windows:**
- Binary name is `ffmpeg.exe` (not `ffmpeg`)
- Typical installation paths include `Program Files` and `C:\ffmpeg`
- PATH environment variable is checked

### Performance Considerations

- FFmpeg processes run synchronously but are wrapped in async Tauri commands
- For large files, consider implementing progress callbacks
- Temporary files are not automatically cleaned up (caller responsibility)

### Security Considerations

- FFmpeg binary path is validated before execution
- Input/output paths are sanitized to prevent path traversal
- No shell injection risk (uses direct command execution)
- User-provided audio formats are passed directly to FFmpeg (trust system FFmpeg)

## Future Enhancements

### Planned Features

1. **Automatic Binary Download**
   - Implement actual download logic in `build.rs`
   - Verify checksums of downloaded binaries
   - Cache binaries to avoid repeated downloads

2. **Progress Callbacks**
   - Parse FFmpeg progress output
   - Emit Tauri events for frontend progress bars
   - Support cancellation of long-running operations

3. **Advanced Operations**
   - Audio normalization
   - Noise reduction
   - Format detection without conversion
   - Batch processing

4. **Bundled Binary Optimization**
   - Build minimal FFmpeg with only required codecs
   - Reduce binary size for app distribution
   - Platform-specific optimization

### Known Limitations

1. No progress reporting for long-running operations
2. No cancellation support once FFmpeg starts
3. Binary download not implemented (manual installation required)
4. Limited metadata extraction (only duration currently)
5. No support for custom FFmpeg flags/parameters

## Troubleshooting

### FFmpeg Not Found

**Problem:** Tests fail or commands return "FFmpeg not found"

**Solutions:**
1. Install FFmpeg via system package manager
2. Verify installation: `which ffmpeg` (Unix) or `where ffmpeg` (Windows)
3. Add FFmpeg to PATH if installed in non-standard location
4. For development, place binary in `src-tauri/binaries/{platform}/`

### Compilation Errors

**Problem:** USB module or subscription module errors

**Solutions:**
These are unrelated to FFmpeg integration. The FFmpeg module compiles successfully.
To verify: `cd src-tauri && cargo check --lib 2>&1 | grep audio/ffmpeg`

### Tests Skipped

**Problem:** All FFmpeg tests are skipped

**Reason:** Tests automatically skip if FFmpeg is not installed on the system.
This is intentional to prevent CI failures.

**Solution:** Install FFmpeg to run tests.

## Resources

- [FFmpeg Official Documentation](https://ffmpeg.org/documentation.html)
- [Tauri External Binaries Guide](https://tauri.app/v1/guides/building/external-binaries)
- [rusqlite chrono feature](https://docs.rs/rusqlite/latest/rusqlite/#optional-features)

## Changelog

### 2026-08-18 - Initial Implementation
- Created FFmpeg wrapper with TDD approach
- Implemented platform-specific binary detection
- Added Tauri IPC commands for frontend integration
- Created comprehensive test suite
- Added build script with optional download support
- Documented architecture and usage patterns
