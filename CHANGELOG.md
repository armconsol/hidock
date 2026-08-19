# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- End-to-end testing with Playwright
- Cross-platform installers (AppImage, deb, rpm, MSI, DMG)
- Code signing and notarization
- App Store deployments
- Performance optimizations for 1000+ notes

## [1.0.0-beta] - 2026-08-19

### 🎉 Major Milestone: 100% Feature Parity Achieved

Complete implementation of all 93 HiNotes API endpoints across 18 functional areas through systematic multi-agent workflow orchestration.

### Added - Authentication & Security
- Google OAuth2 authentication with PKCE (RFC 7636)
- Apple Sign In with form_post response mode and JWT validation
- Email/password authentication with registration
- Secure token storage via OS keyring (Keychain/Credential Manager/Secret Service)
- Automatic token refresh with 5-minute buffer
- Password change and reset flows
- Email verification system
- Account deletion with confirmation

### Added - User Management (17 endpoints)
- User profile management (display name, region, avatar)
- Avatar upload with multipart form support
- Trial system (check eligibility, claim trial)
- Activation code system
- User info endpoint with caching
- Complete security settings UI

### Added - Notes & Whisper Notes
- Whisper notes implementation (7 endpoints)
- Quick voice note recording UI
- Conversion to regular notes, todos, and calendar events
- Whisper list view with timestamps and playback
- Press-and-hold recorder component
- Calendar event extraction from whispers

### Added - Audio Processing
- FFmpeg integration for audio operations
- Audio merge, replace, and extract segment operations
- Audio cache with LRU eviction
- Audio player component with timeline
- Format conversion support
- Progress tracking for audio operations

### Added - Translation (Real API Integration)
- Removed mock HashMap translations
- Integrated HiNotes translation API
- Added whatlang for language detection
- Cache-first strategy for performance
- Offline translation fallback
- Translation quality rating
- Live translation sessions
- Translation segment tracking

### Added - Speaker Diarization (Cloud API)
- Cloud-based speaker detection API integration
- Removed mock segment generation
- Voice signature extraction
- Speaker matching with cosine similarity
- Speaker profile management
- Multi-speaker audio analysis

### Added - Calendar Integration
- Bidirectional Google Calendar sync
- Background sync worker (30-second interval)
- Google Calendar OAuth2 with calendar scope
- Conflict resolution (last-write-wins)
- Recording status notifications to Google Calendar
- Event creation/update/deletion sync
- Calendar event database tracking

### Added - Device Management
- HiDoc P1 USB device detection (VID 0x10d6/0x1395)
- Device file list/download/upload (3 endpoints)
- File transfer with progress tracking
- Device file browser UI component
- Mass storage approach for file access
- Bulk file sync operation
- Audio file import from device

### Added - Subscription Management
- RevenueCat integration (5 endpoints)
- Subscription status with caching (5-minute TTL)
- Grace period logic (7 days)
- Trial check and claim
- Billing portal URL access
- Purchase receipt history
- Plan selector UI with comparison
- Billing history table
- Trial banner with countdown

### Added - Referral Program
- Referral code generation and tracking
- QR code generation for sharing
- Social media sharing (Twitter, Facebook, WhatsApp, Email)
- Referral statistics (signups, conversions, earnings)
- Reward redemption (minutes or cash)
- PayPal integration for payouts
- Reward history tracking
- Complete referral dashboard UI

### Added - Settings & Sync
- Settings cloud sync (4 endpoints)
- Bidirectional sync with conflict resolution
- AI engine selection
- Sync status indicator
- Manual sync trigger
- Auto-sync on: startup, changes (debounced 2s), resume
- Settings: theme, language, transcription_engine, auto_translation, recording_quality, calendar_sync, notifications

### Added - Offline Functionality
- Background sync worker with Tokio
- 6 operation types: notes, folders, todos, calendar, templates, settings
- Retry logic with exponential backoff (max 3 attempts)
- Network connectivity detection
- Pending operations queue in SQLite
- Sync indicator UI
- Last-write-wins conflict resolution

### Added - UI Components (26 new)
- WhisperList, WhisperRecorder, WhisperActions
- PlanSelector, BillingHistory, TrialBanner
- ReferralLink, ReferralStats, RewardsList, PayoutSettings
- DeviceFiles browser with drag-drop upload
- AvatarUpload with crop/preview
- SyncIndicator and SyncButton
- Enhanced Settings page with sync status

### Added - Pages (6 new)
- Whispers page
- Subscription page
- Referrals page  
- Profile page
- Security page
- Enhanced Settings page with cloud sync

### Added - Testing
- 384 frontend tests (100% passing)
- 148+ backend tests (100% passing)
- 532+ total tests
- Comprehensive component test coverage
- API client integration tests
- Mock setup for Tauri commands

### Changed
- API endpoint changed from localhost to production `https://hinotes.hidock.com/v1`
- Translation engine uses real API instead of mock HashMap
- Speaker diarization uses cloud API instead of mock segments
- OAuth handlers use production Google/Apple endpoints
- All database operations use real SQLite (no mocks)

### Fixed
- Translation commands Send bounds issue (moved to managed Tauri state)
- Audio commands module compilation errors
- AppLayout test failures (icon mocks)
- Frontend test compatibility with React 18
- Type mismatches in audio processor
- Iterator issues in speaker commands

### Removed
- Mock translation HashMap implementation
- Mock speaker segment generation
- Commented-out translation commands (now active)
- Placeholder implementations (all replaced with real code)
- Test-only mock implementations from production code

### Documentation
- 20 implementation guides created
- 5 project tickets documented
- 9 status reports generated
- 7 USB protocol analysis documents
- Comprehensive API endpoint documentation
- OAuth setup instructions
- Feature parity completion report
- Reorganized all docs into `docs/` structure

### Performance
- Translation cache reduces API calls
- Audio cache with LRU eviction
- Subscription status caching (5-minute TTL)
- Background workers use efficient polling
- SQLite indexes for fast queries

### Security
- PKCE flow for OAuth2 (RFC 7636)
- State parameter for CSRF protection
- JWT validation for Apple ID tokens
- OS keyring for secure token storage
- Input validation on all API calls
- Error sanitization (no sensitive data in messages)

## [1.0.0-alpha] - 2026-08-18

### Added - Initial Setup
- Project initialization with Tauri 2.0 + React 18 + TypeScript
- SQLite database with comprehensive schema
- 10 functional areas implemented:
  - Notes (CRUD, search, folders, audio attachments)
  - Folders (organization, CRUD)
  - Calendar (CRUD, event sync)
  - To-Do (task management)
  - Templates (CRUD, favorites, defaults)
  - Smart Labels (categories, tags)
  - Vocabulary (custom dictionary, import/export)
  - Devices (binding, status tracking)
  - Audio Processing (basic operations)
  - Note Sharing (share links, access control)
- Initial frontend components (42 components)
- 179 frontend tests passing
- 148 backend tests passing
- Cross-platform build system (Mac, Linux, Windows)
- Documentation structure
- Git repository setup
- MIT License
- README.md with project overview
- CONTRIBUTING.md with TDD workflow
- CHANGELOG.md (this file)

### Added - Build System
- macOS universal binary builder
- Linux multi-format builder (AppImage, deb, rpm)
- Windows MSI builder
- GitHub Actions CI/CD automation
- Code signing setup documentation

### Added - Documentation
- Comprehensive API documentation (93 endpoints)
- OpenAPI 3.0 specification
- Quick reference guide with curl examples
- Python client library
- Project summary and ticket documentation

## Version History

- **1.0.0-beta** (2026-08-19): 100% feature parity, all 93 endpoints, production-ready
- **1.0.0-alpha** (2026-08-18): Initial implementation, 10/18 functional areas

## Migration Guide

### From Alpha to Beta

**New Features:**
- OAuth authentication (Google, Apple) - configure credentials in `.env`
- Whisper notes - access via new Whispers page
- Subscription management - view in new Subscription page
- Referral program - share codes in new Referrals page
- User profile and security - manage in new Profile/Security pages
- Google Calendar sync - enable in Settings
- Settings cloud sync - automatic after login

**Breaking Changes:**
- None - fully backward compatible with alpha database schema

**Configuration Required:**
```env
# Add to .env file
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
```

**Database Migrations:**
- Automatic on first launch
- New tables: calendar_sync, translation_sessions, speaker_segments, referral_codes, rewards
- Existing data preserved

## Support

For issues, feature requests, or questions:
- Gitea Issues: https://gogs.tftsr.com/sarman/hinotes/issues
- Repository: https://gogs.tftsr.com/sarman/hinotes
- Documentation: [docs/](docs/)
- API Reference: [API_Notes/](API_Notes/)

---

*Last Updated: 2026-08-19*
