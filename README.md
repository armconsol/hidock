# HiNotes Desktop

> Cross-platform desktop application for HiNotes with offline-first architecture

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue)](https://react.dev/)

## Overview

HiNotes Desktop is an unofficial cross-platform desktop application for the HiNotes transcription service, designed to work seamlessly with the HiDoc P1 USB audio transcription device. This application provides full feature parity with the HiNotes web application while adding critical offline functionality.

### Key Features

- ✅ **Offline-First**: Create, edit, and organize notes without an internet connection
- 🌐 **Cross-Platform**: Native applications for macOS, Linux, and Windows
- 🎨 **Customizable Dashboard**: Drag-and-drop widget system for personalized layouts
- 🌓 **Light & Dark Themes**: Built-in theme switching
- 🔄 **Automatic Sync**: Background synchronization when online
- 🎙️ **HiDoc P1 Integration**: Direct USB device support for audio file transfers
- 🎵 **Audio Management**: Built-in audio player with merge, replace, and editing capabilities
- 📋 **Full Feature Parity**: 90+ API endpoints implemented across 18 functional areas

## Quick Start

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Run tests
npm test
cargo test

# Build for production
npm run tauri build
```

## Development

This project follows strict Test-Driven Development (TDD) methodology. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) file for details.

Copyright (c) 2026 Shaun Arman

## Disclaimer

This is an **unofficial, community-developed application**. It is not created, endorsed, or supported by HiDock. Use at your own risk.
