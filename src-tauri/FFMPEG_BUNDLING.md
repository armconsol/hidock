# FFmpeg Bundling Instructions

The audio editing features require FFmpeg to be available. There are two options:

## Option 1: System FFmpeg (Development)

Install FFmpeg on your system:

### macOS
```bash
brew install ffmpeg
```

### Windows
Download from https://ffmpeg.org/download.html or use:
```powershell
choco install ffmpeg
```

### Linux
```bash
# Debian/Ubuntu
sudo apt install ffmpeg

# Fedora
sudo dnf install ffmpeg

# Arch
sudo pacman -S ffmpeg
```

## Option 2: Bundle FFmpeg (Production Builds)

To bundle FFmpeg with the application:

1. Create `src-tauri/bin/` directory
2. Download FFmpeg static binaries for each target platform:
   - macOS (ARM64): `bin/ffmpeg-aarch64-apple-darwin`
   - macOS (Intel): `bin/ffmpeg-x86_64-apple-darwin`
   - Windows: `bin/ffmpeg-x86_64-pc-windows-msvc.exe`
   - Linux: `bin/ffmpeg-x86_64-unknown-linux-gnu`

3. Update `tauri.conf.json`:
```json
{
  "bundle": {
    "externalBin": ["bin/ffmpeg"],
    "resources": ["bin/ffmpeg*"]
  }
}
```

### Download Sources

- **macOS/Linux**: https://evermeet.cx/ffmpeg/ (macOS) or https://johnvansickle.com/ffmpeg/ (Linux)
- **Windows**: https://www.gyan.dev/ffmpeg/builds/

### Note on Binary Names

When bundling, Tauri automatically appends the target triple to binary names:
- `bin/ffmpeg` becomes `bin/ffmpeg-aarch64-apple-darwin` on ARM64 macOS
- `bin/ffmpeg` becomes `bin/ffmpeg-x86_64-pc-windows-msvc.exe` on Windows
- etc.

The `AudioProcessor` will automatically detect and use bundled binaries when available, falling back to system FFmpeg otherwise.

## Verification

Test FFmpeg availability:
```typescript
import { invoke } from '@tauri-apps/api/core';

const isAvailable = await invoke('verify_ffmpeg');
console.log('FFmpeg available:', isAvailable);
```
