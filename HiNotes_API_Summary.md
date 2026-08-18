# HiNotes API Mapping - Summary

## Project Overview
Successfully reverse-engineered the HiNotes web application API for the HiDoc P1 USB audio transcription device.

## Files Created
1. **HiNotes_API_Documentation.md** - Comprehensive API documentation with 90+ endpoints
2. **HiNotes_OpenAPI.yaml** - OpenAPI 3.0 specification for API integration
3. **HiNotes_API_Summary.md** - This summary document

## Key Findings

### Base URL
```
https://hinotes.hidock.com/v1
```

### Total Endpoints Discovered
**90+ API endpoints** across 15 functional categories

### Major API Categories

1. **Authentication (4 endpoints)**
   - Google OAuth2
   - Apple OAuth2
   - Email/Password Sign In
   - User Registration

2. **User Management (17 endpoints)**
   - Profile management
   - Email verification
   - Password reset
   - Avatar upload
   - Account deletion

3. **Device Management (9 endpoints)**
   - HiDoc P1 device binding/unbinding
   - Device file management
   - Device status monitoring
   - Accessibility settings

4. **Notes Management (12 endpoints)**
   - Recording notes list/CRUD
   - Whisper notes (quick voice notes)
   - Speaker identification
   - Note rating/feedback
   - Convert notes to todos/calendar

5. **Audio Operations (3 endpoints)**
   - Merge audio files
   - Replace audio
   - Save as new note

6. **Folder Organization (4 endpoints)**
   - Create/rename/delete folders
   - Assign notes to folders

7. **To-Do Management (5 endpoints)**
   - List/create/update/delete todos
   - Due date management
   - Smart labels

8. **Calendar Integration (4 endpoints)**
   - List calendar events
   - Add events
   - OAuth2 authorization
   - Device state notifications

9. **Live Translation (3 endpoints)**
   - Supported languages
   - Active translation sessions
   - Translation rating

10. **Templates (9 endpoints)**
    - List/create/update/delete templates
    - Favorite templates
    - Default templates

11. **Smart Labels (3 endpoints)**
    - Create/update/delete labels

12. **Custom Vocabulary (3 endpoints)**
    - Add custom words for transcription accuracy

13. **User Settings (4 endpoints)**
    - Get/save preferences
    - AI engine selection

14. **Sharing (2 endpoints)**
    - Create share links
    - List shared transcriptions

15. **Subscription & Billing (3 endpoints)**
    - Subscriber info
    - Receipts
    - RevenueCat integration

16. **Referral Program (7 endpoints)**
    - Rewards overview
    - PayPal integration
    - Referral messages

## Technical Architecture

### Frontend Stack
- **Framework**: React
- **Build Tool**: Vite
- **UI Library**: Arco Design
- **State Management**: Custom state solution
- **i18n**: Multilingual support (English, Chinese, Japanese)
- **Audio Processing**: FFmpeg WebAssembly

### Authentication
- OAuth2 (Google, Apple)
- Traditional email/password
- Token-based authentication (Bearer tokens)

### Third-Party Integrations
- **RevenueCat**: Subscription management
- **Google APIs**: Calendar, Drive, OAuth
- **Apple ID**: Authentication
- **Google reCAPTCHA**: Security
- **PayPal**: Referral payouts

### Data Patterns
- **Date/Time Format**: `YYYY-MM-DD HH:MM:SS`
- **Pagination**: `pageIndex` (0-based), `pageSize`
- **Sorting**: `sortField`, `sortType` (asc/desc)
- **Timezone**: Offset in minutes from UTC

### Observed API Behavior
- Most endpoints use POST even for read operations
- Frequent polling of calendar events
- Real-time sync via `/v1/changes` endpoint
- Initial app state via `/v1/entry/info`

## Key Endpoints for HiDoc P1 Integration

### Device Communication
```
POST /v1/user/device/bind           # Connect HiDoc P1
POST /v1/user/device/list           # List connected devices
GET  /v1/user/device/status         # Monitor connection
GET  /v1/user/device/file/list      # List recordings
POST /v1/user/device/file/upload    # Transfer audio
```

### Recording & Transcription
```
GET  /v1/note/recording/list        # List transcribed notes
POST /v1/audio/merge                # Combine recordings
POST /v1/audio/saveAsNew            # Create note from audio
POST /v1/note/speaker/find          # Identify speakers
```

### Live Translation
```
GET  /v1/live/language/list         # Supported languages
GET  /v1/live/note/get              # Active session
POST /v1/live/rate                  # Feedback
```

## Network Traffic Observations

### High-Frequency Calls
- `/v1/calendar/event/list` - Called repeatedly (polling)
- `/v1/user/info` - Multiple times per session
- `/v1/changes` - Periodic sync

### Initial Load Sequence
1. OAuth2 sign in
2. Get user info
3. List devices
4. Get referral rewards
5. Get entry info
6. Sync changes
7. List notes/whispers/todos
8. Load calendar events

## Missing Information

To complete the API documentation, the following is needed:

1. **Request/Response Bodies**: Actual JSON structures
2. **Authentication Headers**: Token format and header names
3. **Error Responses**: Error codes and messages
4. **Rate Limits**: Request throttling policies
5. **WebSocket Endpoints**: Real-time communication protocol
6. **File Upload Formats**: Audio file specifications
7. **HiDoc P1 USB Protocol**: Low-level device communication

## Recommendations for API Usage

### Best Practices
1. **Implement Token Refresh**: Handle authentication expiration
2. **Rate Limiting**: Add delays between requests
3. **Error Handling**: Implement retry logic with exponential backoff
4. **Caching**: Cache frequently accessed data (folders, settings)
5. **Batch Operations**: Group related requests when possible

### Security Considerations
1. Store authentication tokens securely
2. Use HTTPS for all requests
3. Implement CORS properly if building web app
4. Don't expose API keys in client-side code
5. Respect user privacy when handling transcriptions

### Testing Strategy
1. Start with read-only endpoints (GET requests)
2. Test authentication flow thoroughly
3. Use staging/test device if available
4. Monitor rate limits during testing
5. Implement comprehensive error logging

## Legal & Ethical Considerations

⚠️ **IMPORTANT DISCLAIMERS**:

1. This is **unofficial** documentation created through reverse engineering
2. Using this API may **violate HiNotes Terms of Service**
3. The API structure may **change without notice**
4. No warranty or support is provided
5. Use at your own risk
6. Always respect user privacy and data protection laws
7. Consider contacting HiDock for official API access

## Next Steps

### For Personal Use
1. Implement authentication flow
2. Test device connection endpoints
3. Build simple note retrieval tool
4. Add error handling and logging

### For Production Use
1. **Contact HiDock/HiNotes** for official API access
2. Request API documentation and terms
3. Negotiate rate limits and SLA
4. Implement OAuth2 properly
5. Set up monitoring and alerting

### For Further Reverse Engineering
1. Capture request/response with Charles Proxy or mitmproxy
2. Analyze WebSocket traffic (if used)
3. Decompile mobile apps for additional endpoints
4. Study USB communication protocol for HiDoc P1
5. Map out data models and relationships

## Tools Used
- **Playwright Browser Automation**: Network traffic capture
- **curl**: JavaScript analysis
- **grep**: Endpoint extraction
- **Browser DevTools**: API observation

## Capture Methodology
1. Navigated to HiNotes web app via Playwright
2. Authenticated with Google OAuth2
3. Captured network traffic during normal usage
4. Extracted JavaScript bundles
5. Pattern-matched API endpoints from source code
6. Documented observed request patterns

## Contact & Contribution
This documentation was created for educational purposes. If you have additional information about the HiNotes API or find errors in this documentation, contributions are welcome.

---

**Created**: August 18, 2026  
**Tool**: Claude Code (Anthropic)  
**Method**: Reverse Engineering via Network Traffic Analysis  
**Status**: Unofficial / Community Documentation
