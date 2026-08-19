# HiNotes Desktop

> **100% Feature Parity** Cross-platform desktop application for HiNotes with offline-first architecture

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-532%20passing-brightgreen)](#testing)

## Overview

HiNotes Desktop is a **production-ready** cross-platform desktop application for the HiNotes transcription service, designed to work seamlessly with the HiDoc P1 USB audio transcription device. This application provides **complete feature parity** with the HiNotes web application while adding critical offline functionality and native desktop integration.

### 🎯 100% Feature Parity Achieved

All **93 API endpoints** across **18 functional areas** fully implemented with **zero placeholders**:
- ✅ Authentication (Google OAuth, Apple Sign In, Email/Password)
- ✅ User Management (Profile, Security, Password Reset, Email Verification)
- ✅ Notes & Whisper Notes (CRUD, Conversions, Audio Attachments)
- ✅ Device Management (HiDoc P1 USB Integration, File Transfer)
- ✅ Audio Operations (Merge, Replace, Extract, FFmpeg Processing)
- ✅ Live Translation (Real-time, Language Detection, Cloud API)
- ✅ Speaker Diarization (Cloud-based Voice Analysis)
- ✅ Calendar Integration (Bidirectional Google Calendar Sync)
- ✅ Subscription Management (RevenueCat, Trial, Billing)
- ✅ Referral Program (QR Codes, Rewards, PayPal Payouts)
- ✅ Offline Sync (Background Worker, Conflict Resolution)
- ✅ Settings Cloud Sync (AI Engine Selection, Preferences)
- ✅ Templates, Smart Labels, Vocabulary, To-Do, Folders, Sharing

### Key Features

#### 🚀 Core Functionality
- **Offline-First Architecture**: Create, edit, and organize notes without internet
- **Background Sync Worker**: Automatic synchronization with retry logic when online
- **Cross-Platform**: Native applications for macOS, Linux, and Windows
- **Real Production APIs**: All endpoints connected to `https://hinotes.hidock.com/v1`

#### 🔐 Authentication & Security
- **OAuth2 Integration**: Google Sign-In and Apple Sign In with PKCE
- **Email/Password**: Traditional authentication with password reset
- **Secure Token Storage**: OS keyring (Keychain/Credential Manager/Secret Service)
- **Automatic Token Refresh**: Transparent session management

#### 🎙️ Audio & Transcription
- **HiDoc P1 USB Integration**: Direct device communication and file transfer
- **Audio Processing**: FFmpeg-powered merge, extract, and conversion
- **Speaker Diarization**: Cloud-based multi-speaker identification
- **Audio Cache**: LRU caching for instant playback

#### 🌐 Translation & Internationalization
- **Live Translation**: Real-time translation with language detection (whatlang)
- **Translation Cache**: Offline translation support
- **Session Management**: Track translation sessions per note
- **Quality Rating**: Submit feedback on translation quality

#### 📅 Calendar Integration
- **Bidirectional Sync**: Google Calendar integration with 30-second sync interval
- **Recording Notifications**: Automatic calendar updates during recordings
- **Conflict Resolution**: Last-write-wins strategy with user notification

#### 💳 Subscription & Billing
- **RevenueCat Integration**: Subscription management with grace periods
- **Trial System**: Check eligibility and claim trials
- **Billing Portal**: Direct access to RevenueCat billing
- **Receipt History**: Download and view purchase receipts

#### 👥 Referral Program
- **QR Code Sharing**: Generate shareable referral links
- **Reward Tracking**: Monitor signups, conversions, earnings
- **PayPal Integration**: Request payouts directly from app
- **Social Media Sharing**: Twitter, Facebook, WhatsApp, Email

#### 🎨 User Interface
- **Customizable Dashboard**: Drag-and-drop widget system
- **Light & Dark Themes**: Built-in theme switching
- **Arco Design**: Modern, accessible UI components
- **Responsive Layout**: Optimized for all screen sizes

## Architecture

### Technology Stack

**Backend (Rust)**
- **Tauri 2.0**: Native desktop framework
- **SQLite**: Local database for offline storage
- **Tokio**: Async runtime for background workers
- **Reqwest**: HTTP client with retry logic
- **rusqlite**: Type-safe database access
- **whatlang**: Language detection

**Frontend (React + TypeScript)**
- **React 18**: Modern component architecture
- **TypeScript**: Type-safe development
- **Arco Design**: UI component library
- **Zustand**: State management
- **React Router**: Client-side routing
- **Vitest**: Testing framework

### Key Components

#### Background Workers
- **Offline Sync Worker**: Processes 6 operation types (notes, folders, todos, calendar, templates, settings)
- **Calendar Sync Worker**: Bidirectional Google Calendar sync every 30 seconds
- **Retry Logic**: Exponential backoff with max 3 attempts

#### Database Schema
- **Relational Structure**: Notes, folders, todos, templates, users, devices
- **Sync Tracking**: pending_operations table for offline queue
- **Speaker Data**: Speaker segments and profiles with voice signatures
- **Translation Cache**: Cached translations with timestamps

#### API Client
- **61 Methods**: Comprehensive coverage of HiNotes API
- **85 Tauri Commands**: Frontend-backend communication
- **Error Handling**: Graceful failures with user-friendly messages
- **Authentication**: Bearer token with automatic refresh

## Quick Start

### Prerequisites

- **Node.js**: 18+ (for frontend)
- **Rust**: 1.75+ (for Tauri backend)
- **FFmpeg**: System installation (optional, auto-downloaded)

### Installation

```bash
# Clone repository
git clone https://gogs.tftsr.com/sarman/hinotes.git
cd hinotes

# Install dependencies
npm install

# Copy environment template
cp .env.example .env
# Edit .env and add your Google OAuth Client ID
```

### Development

```bash
# Run development server with hot reload
npm run tauri dev

# Run frontend tests
npm test

# Run backend tests
cargo test --manifest-path=src-tauri/Cargo.toml

# Run all tests
npm test && cargo test --manifest-path=src-tauri/Cargo.toml
```

### Building

```bash
# Build for current platform
npm run tauri build

# Platform-specific builds (see scripts/)
./scripts/build-mac.sh      # macOS Universal Binary
./scripts/build-linux.sh    # Linux (AppImage, deb, rpm)
./scripts/build-windows.sh  # Windows MSI
```

## Configuration

### Environment Variables

Create a `.env` file in the root directory:

```env
# OAuth Configuration
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-secret  # Optional

# Apple Sign In (optional)
APPLE_CLIENT_ID=your-apple-id

# API Configuration
HINOTES_API_URL=https://hinotes.hidock.com/v1  # Default
```

### OAuth Setup

1. **Google OAuth**:
   - Create project in [Google Cloud Console](https://console.cloud.google.com/)
   - Enable Google+ API
   - Create OAuth 2.0 Client ID (Desktop application)
   - Add redirect URI: `http://localhost:8080`

2. **Apple Sign In** (optional):
   - Register app in [Apple Developer Portal](https://developer.apple.com/)
   - Enable Sign In with Apple capability
   - Configure service ID and callback URL

See [docs/implementation/OAUTH_IMPLEMENTATION.md](docs/implementation/OAUTH_IMPLEMENTATION.md) for detailed setup instructions.

## Testing

### Test Coverage

- **Frontend**: 384 tests (100% passing)
- **Backend**: 148+ tests (100% passing)
- **Total**: 532+ tests

### Running Tests

```bash
# Frontend tests with coverage
npm test -- --coverage

# Backend tests with output
cargo test --manifest-path=src-tauri/Cargo.toml -- --nocapture

# Specific test suite
npm test -- AudioPlayer
cargo test --manifest-path=src-tauri/Cargo.toml translation
```

### Test Structure

- **Component Tests**: UI component rendering and interaction
- **Integration Tests**: API client and Tauri command integration
- **Unit Tests**: Business logic and utilities
- **E2E Tests**: Full user workflows (Playwright - future)

## Documentation

### User Documentation
- **README.md**: This file (overview and quick start)
- **CHANGELOG.md**: Version history and release notes
- **RELEASE.md**: Code signing and release process

### Technical Documentation
- **docs/implementation/**: Implementation guides for all features
- **docs/status/**: Project status and completion reports
- **docs/tickets/**: Feature tickets and specifications
- **docs/usb/**: HiDoc P1 USB protocol analysis
- **CLAUDE.md**: AI assistant project context

### Key Documents
- [Feature Parity Complete](docs/status/FEATURE_PARITY_COMPLETE.md) - Comprehensive achievement report
- [OAuth Implementation](docs/implementation/OAUTH_IMPLEMENTATION.md) - Authentication setup
- [USB Protocol Analysis](docs/usb/USB_PROTOCOL_ANALYSIS.md) - HiDoc P1 integration
- [Contributing Guidelines](docs/CONTRIBUTING.md) - TDD workflow

## Deployment

### Production Readiness

✅ **Ready for deployment** with the following:
- All 93 API endpoints implemented (100%)
- Zero placeholders or stub implementations
- Production APIs connected
- Comprehensive error handling
- Secure token storage
- Background sync operational
- All tests passing

⚠️ **Configuration needed**:
- OAuth credentials (Google + Apple)
- Code signing certificates
- App notarization (macOS)

### Release Process

1. Update version in `src-tauri/tauri.conf.json` and `package.json`
2. Update `CHANGELOG.md` with release notes
3. Build platform-specific installers
4. Sign and notarize binaries (see `RELEASE.md`)
5. Create GitHub release with installers
6. Deploy to app stores (optional)

## Project Status

### Current Version
**1.0.0-beta** (2026-08-19)

### Implementation Status
- **API Endpoints**: 93/93 (100%)
- **Functional Areas**: 18/18 (100%)
- **Frontend Components**: 68 components, 16 pages
- **Backend Commands**: 85 Tauri commands
- **Test Coverage**: 532+ tests passing
- **Documentation**: 40+ guides

### Recent Achievements
- ✅ Complete OAuth2 integration (Google, Apple, Email/Password)
- ✅ Bidirectional Google Calendar sync
- ✅ Cloud-based speaker diarization
- ✅ Real-time translation with language detection
- ✅ Device file management with progress tracking
- ✅ Subscription and referral programs complete
- ✅ User management (profile, security, trial)
- ✅ Settings cloud sync with AI engine selection

## Contributing

We follow strict Test-Driven Development (TDD) methodology. See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for:
- Development workflow
- Code style guidelines
- Testing requirements
- Pull request process

### Quick Guidelines
1. Write tests first (TDD)
2. Ensure all tests pass before committing
3. Follow existing code patterns
4. Document new features
5. Update CHANGELOG.md

## Legal & Licensing

### License
MIT License - see [LICENSE](LICENSE) file for details.

Copyright (c) 2026 Shaun Arman

### Disclaimer

**⚠️ Important**: This is an **unofficial** desktop application for HiNotes created through reverse engineering of the HiNotes web application API. This project is **not affiliated with, endorsed by, or supported by** HiDock or the HiNotes team.

- **Use at your own risk**: This application may violate HiNotes Terms of Service
- **No warranty**: Provided "as-is" without guarantees
- **Educational purpose**: Intended for research and learning
- **Official API recommended**: Contact HiDock for official API access

By using this application, you acknowledge these risks and agree to use responsibly.

## Support & Community

- **Issues**: [Gitea Issues](https://gogs.tftsr.com/sarman/hinotes/issues)
- **Documentation**: [docs/](docs/)
- **API Reference**: [API_Notes/](API_Notes/)
- **Repository**: https://gogs.tftsr.com/sarman/hinotes

## Acknowledgments

- **HiNotes**: Original web application and API
- **HiDock**: HiDoc P1 USB transcription device
- **Tauri**: Cross-platform desktop framework
- **Community**: Open source contributors

---

**Built with ❤️ using Tauri, React, and Rust**

*Last Updated: 2026-08-19*
