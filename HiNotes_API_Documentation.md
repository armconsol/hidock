# HiNotes API Documentation

**Base URL**: `https://hinotes.hidock.com`

**API Version**: v1

This is an unofficial API mapping for the HiNotes web application, which is used with the HiDoc P1 USB audio transcription device.

## Authentication

### OAuth2 - Google Sign In
- **Endpoint**: `POST /v1/oauth2/signin/google`
- **Description**: Authenticate user via Google OAuth2
- **Observed in network traffic**

### OAuth2 - Apple Sign In
- **Endpoint**: `POST /v1/oauth2/signin/apple`
- **Description**: Authenticate user via Apple OAuth2

### User Sign In
- **Endpoint**: `POST /v1/user/signin`
- **Description**: Direct email/password authentication

### User Register
- **Endpoint**: `POST /v1/user/register`
- **Description**: Create a new user account

### User Logout
- **Endpoint**: `POST /v1/user/logout`
- **Description**: End user session

## User Management

### Get User Info
- **Endpoint**: `POST /v1/user/info`
- **Description**: Retrieve current user information
- **Observed frequency**: Multiple calls during session

### Update User Profile
- **Endpoint**: `POST /v1/user/rename`
- **Description**: Update user display name

### Update User Region
- **Endpoint**: `POST /v1/user/region/update`
- **Description**: Update user's region/locale settings

### Update User Role
- **Endpoint**: `POST /v1/user/role/update`
- **Description**: Update user role/permissions

### Upload Avatar
- **Endpoint**: `POST /v1/user/avatar/upload`
- **Description**: Upload user profile picture

### Update Password
- **Endpoint**: `POST /v1/user/password/update`
- **Description**: Change user password

### Delete User Account
- **Endpoint**: `POST /v1/user/delete`
- **Description**: Permanently delete user account

### Email Verification
- **Endpoint**: `POST /v1/user/email/verification/send`
- **Description**: Send email verification code

- **Endpoint**: `POST /v1/user/email/verification/verify`
- **Description**: Verify email with code

### Password Reset
- **Endpoint**: `POST /v1/user/reset/authcode/send`
- **Description**: Send password reset code

- **Endpoint**: `POST /v1/user/reset/check`
- **Description**: Verify reset code

- **Endpoint**: `POST /v1/user/reset/save`
- **Description**: Save new password after reset

### Country List
- **Endpoint**: `GET /v1/user/country/list`
- **Description**: Get list of supported countries

### Trial Management
- **Endpoint**: `GET /v1/user/trial/check`
- **Description**: Check trial status

- **Endpoint**: `POST /v1/user/trial/claim`
- **Description**: Claim trial subscription

### Activation Code
- **Endpoint**: `POST /v1/user/activateCode/send`
- **Description**: Send activation code

## Device Management

### List Devices
- **Endpoint**: `POST /v1/user/device/list`
- **Description**: Get list of connected HiDoc devices
- **Observed in network traffic**

### Bind Device
- **Endpoint**: `POST /v1/user/device/bind`
- **Description**: Connect a new HiDoc device to account

### Unbind Device
- **Endpoint**: `POST /v1/user/device/unbind`
- **Description**: Disconnect a HiDoc device from account

### Rename Device
- **Endpoint**: `POST /v1/user/device/rename`
- **Description**: Change device display name

### Device Status
- **Endpoint**: `GET /v1/user/device/status`
- **Description**: Get current device connection status

### Device Accessibility Settings
- **Endpoint**: `POST /v1/user/device/accessibility/set`
- **Description**: Configure device accessibility features

### Device File Management
- **Endpoint**: `GET /v1/user/device/file/list`
- **Description**: List files on connected device

- **Endpoint**: `GET /v1/user/device/file/get`
- **Description**: Download file from device

- **Endpoint**: `POST /v1/user/device/file/upload`
- **Description**: Upload file to device

## Notes Management

### List Recording Notes
- **Endpoint**: `GET /v1/note/recording/list`
- **Query Parameters**:
  - `folderId`: Filter by folder ID (-1 for all)
  - `pageIndex`: Page number (0-indexed)
  - `pageSize`: Items per page (observed: 10, 20)
  - `sortType`: Sort direction (`desc`, `asc`)
  - `sortField`: Sort by field (`createtime`)
- **Description**: Get list of recorded/transcribed notes
- **Example**: `/v1/note/recording/list?folderId=-1&pageIndex=0&pageSize=20&sortType=desc&sortField=createtime`
- **Observed in network traffic**

### List Whisper Notes
- **Endpoint**: `GET /v1/note/whisper/list`
- **Query Parameters**:
  - `pageSize`: Items per page (observed: 20)
  - `sortField`: Sort by field (`create_time`)
- **Description**: Get list of "whisper" notes (quick voice notes)
- **Example**: `/v1/note/whisper/list?pageSize=20&sortField=create_time`
- **Observed in network traffic**

### Delete Note
- **Endpoint**: `POST /v1/note/delete`
- **Description**: Delete a note

### Rate Note
- **Endpoint**: `POST /v1/note/rate`
- **Description**: Provide feedback/rating for transcription quality

### Find Speakers
- **Endpoint**: `POST /v1/note/speaker/find`
- **Description**: Identify speakers in multi-person recordings

### Whisper Note Operations
- **Endpoint**: `POST /v1/note/whisper/add/todo`
- **Description**: Convert whisper note to todo item

- **Endpoint**: `POST /v1/note/whisper/create/note`
- **Description**: Convert whisper to full note

- **Endpoint**: `POST /v1/note/whisper/extract/calendar`
- **Description**: Extract calendar events from whisper note

- **Endpoint**: `POST /v1/note/whisper/paragraph/update`
- **Description**: Edit paragraph in whisper note

## Audio Management

### Merge Audio
- **Endpoint**: `POST /v1/audio/merge`
- **Description**: Combine multiple audio files

### Replace Audio
- **Endpoint**: `POST /v1/audio/replace`
- **Description**: Replace audio in existing note

### Save Audio As New
- **Endpoint**: `POST /v1/audio/saveAsNew`
- **Description**: Create new note from audio segment

## Folder Management

### List Folders
- **Endpoint**: `POST /v1/folder/list`
- **Description**: Get user's folder structure
- **Observed in network traffic**

### Create Folder
- **Endpoint**: `POST /v1/folder/create`
- **Description**: Create new folder

### Rename Folder
- **Endpoint**: `POST /v1/folder/rename`
- **Description**: Change folder name

### Remove Folder
- **Endpoint**: `POST /v1/folder/remove`
- **Description**: Delete folder

### Assign to Folder
- **Endpoint**: `POST /v1/folder/assign`
- **Description**: Move note to folder

## To-Do Management

### List To-Dos
- **Endpoint**: `GET /v1/todo/list`
- **Query Parameters**:
  - `pageSize`: Items per page (observed: 10)
  - `state`: Filter by state (`open`, `closed`)
  - `tzOffset`: Timezone offset in minutes (observed: 300)
  - `dueDateStart`: Start date filter (format: `YYYY-MM-DD HH:MM:SS`)
  - `dueDateEnd`: End date filter (format: `YYYY-MM-DD HH:MM:SS`)
- **Description**: Get user's to-do items
- **Example**: `/v1/todo/list?pageSize=10&state=open&tzOffset=300&dueDateStart=2026-08-18 00:00:00&dueDateEnd=2026-08-19 00:00:00`
- **Observed in network traffic**

### Update To-Do Description
- **Endpoint**: `POST /v1/todo/update/description`
- **Description**: Edit to-do item text

### Update To-Do Due Date
- **Endpoint**: `POST /v1/todo/update/dueDate`
- **Description**: Change to-do deadline

### Update To-Do Smart Label
- **Endpoint**: `POST /v1/todo/update/smartLabel`
- **Description**: Assign smart label/category

### Delete To-Do
- **Endpoint**: `POST /v1/todo/delete`
- **Description**: Remove to-do item

## Calendar Integration

### List Calendar Events
- **Endpoint**: `GET /v1/calendar/event/list`
- **Query Parameters**:
  - `start_time`: Start date (format: `YYYY-MM-DD HH:MM:SS`)
  - `end_time`: End date (format: `YYYY-MM-DD HH:MM:SS`)
  - `tz_offset`: Timezone offset in minutes (observed: 300)
- **Description**: Get calendar events in date range
- **Example**: `/v1/calendar/event/list?start_time=2026-08-01 00:00:00&end_time=2026-08-31 23:59:59&tz_offset=300`
- **Observed in network traffic** (called frequently)

### Add Calendar Event
- **Endpoint**: `POST /v1/calendar/event/add`
- **Description**: Create new calendar event

### Calendar OAuth2 Authorization
- **Endpoint**: `GET /v1/calendar/oauth2/authorize`
- **Description**: Authorize calendar integration (Google Calendar, etc.)

### Device State Notice
- **Endpoint**: `POST /v1/calendar/event/device_state/notice`
- **Description**: Notify calendar of device recording state

## Live Translation

### Get Language List
- **Endpoint**: `GET /v1/live/language/list`
- **Description**: Get supported languages for live translation

### Get Live Note
- **Endpoint**: `GET /v1/live/note/get`
- **Description**: Retrieve active live translation session

### Rate Live Translation
- **Endpoint**: `POST /v1/live/rate`
- **Description**: Provide feedback on translation quality

## Templates

### List Templates
- **Endpoint**: `GET /v1/template/list`
- **Description**: Get available note templates

### Get Template Info
- **Endpoint**: `GET /v1/template/info`
- **Description**: Get template details

### Save Template
- **Endpoint**: `POST /v1/template/save?createNew=true`
- **Description**: Create or save template

### Update Template Content
- **Endpoint**: `POST /v1/template/content/update`
- **Description**: Edit template content

### Update Template Title
- **Endpoint**: `POST /v1/template/title/update`
- **Description**: Change template name

### Delete Template
- **Endpoint**: `POST /v1/template/delete`
- **Description**: Remove template

### Toggle Template Favorite
- **Endpoint**: `POST /v1/template/fav/toggle`
- **Description**: Add/remove from favorites

### Set Default Template
- **Endpoint**: `POST /v1/template/setDefault`
- **Description**: Set as default template

### Test Template
- **Endpoint**: `POST /v1/template/test`
- **Description**: Preview template rendering

## Smart Labels

### Create Smart Label
- **Endpoint**: `POST /v1/smart_label/create`
- **Description**: Create new smart label/category

### Update Smart Label
- **Endpoint**: `POST /v1/smart_label/update`
- **Description**: Edit smart label

### Delete Smart Label
- **Endpoint**: `POST /v1/smart_label/delete`
- **Description**: Remove smart label

## Vocabulary

### List Vocabulary
- **Endpoint**: `GET /v1/vocabulary/list`
- **Description**: Get custom vocabulary for transcription

### Create Vocabulary Entry
- **Endpoint**: `POST /v1/vocabulary/create`
- **Description**: Add word to custom vocabulary

### Delete Vocabulary Entry
- **Endpoint**: `POST /v1/vocabulary/delete`
- **Description**: Remove word from vocabulary

## User Settings

### Get Settings
- **Endpoint**: `POST /v1/user/setting/get`
- **Description**: Retrieve user preferences
- **Observed in network traffic**

### List Settings
- **Endpoint**: `GET /v1/user/setting/list`
- **Description**: Get all available settings

### Save Settings
- **Endpoint**: `POST /v1/user/setting/save`
- **Description**: Update user preferences

### List AI Engines
- **Endpoint**: `GET /v1/user/setting/ai_engine/list`
- **Description**: Get available AI transcription engines

## Sharing

### Create Share Link
- **Endpoint**: `POST /v1/share/create`
- **Description**: Generate shareable link for note

### List Shared Transcriptions
- **Endpoint**: `GET /v1/share/transcription/list?shortId=`
- **Description**: Get transcriptions from share link

## Subscription & Billing

### Get Subscriber Info
- **Endpoint**: `GET /v1/subscribers`
- **Description**: Get subscription details

### Get Receipts
- **Endpoint**: `GET /v1/receipts`
- **Description**: Retrieve purchase receipts

### Revenue Cat Portal
- **Endpoint**: `GET /v1/payment/rc/portal`
- **Description**: Access RevenueCat billing portal

**Note**: The app uses RevenueCat for subscription management:
- RevenueCat API: `https://api.revenuecat.com`
- Subscription products include monthly/yearly plans and quota packages

## Referral Program

### Referral Overview
- **Endpoint**: `GET /v1/referral/overview`
- **Description**: Get referral program details

### Rewards Overview
- **Endpoint**: `GET /v1/referral/rewards-overview`
- **Description**: Get user's referral rewards
- **Observed in network traffic**

### Choose Minutes Reward
- **Endpoint**: `POST /v1/referral/choose-minutes`
- **Description**: Claim transcription minutes as reward

### Get Message Template
- **Endpoint**: `GET /v1/referral/message-template`
- **Description**: Get referral message templates

### PayPal Connection
- **Endpoint**: `POST /v1/referral/paypal/connect`
- **Description**: Connect PayPal for referral payouts

- **Endpoint**: `POST /v1/referral/paypal/disconnect`
- **Description**: Disconnect PayPal account

## Redemption

### Get Redemption Info
- **Endpoint**: `GET /v1/redemption/info`
- **Description**: Get redemption details

### Fulfill Redemption
- **Endpoint**: `POST /v1/redemption/fulfill`
- **Description**: Redeem code or reward

## Sync & Changes

### Get Entry Info
- **Endpoint**: `POST /v1/entry/info`
- **Description**: Get application entry point data
- **Observed in network traffic**

### Sync Changes
- **Endpoint**: `POST /v1/changes`
- **Description**: Synchronize local changes with server
- **Observed in network traffic**

## Technical Notes

### Base Architecture
- **Frontend**: React-based SPA (Single Page Application)
- **Build Tool**: Vite (based on asset filenames)
- **UI Framework**: Arco Design
- **State Management**: Custom state management (vendor-state bundle)
- **Internationalization**: i18n support (en, zh, ja)
- **Audio Processing**: FFmpeg WebAssembly (`/ffmpeg/ffmpeg-core.wasm`)
- **Integrations**: Google Drive, Google Calendar, Apple Sign In

### Authentication Flow
1. User signs in via Google/Apple OAuth2 or email/password
2. Authentication token stored (likely in localStorage or cookies)
3. Token included in subsequent API requests (likely as Bearer token in Authorization header)

### Data Synchronization
- The `/v1/changes` endpoint is called periodically for sync
- The `/v1/entry/info` provides initial app state

### Third-Party Services
- **RevenueCat**: Subscription and billing management
- **Google APIs**: Calendar integration, Drive storage, OAuth
- **Apple ID**: Authentication
- **Google reCAPTCHA**: Login protection

### Observed Patterns
- Most endpoints use POST even for read operations
- Date/time formats: `YYYY-MM-DD HH:MM:SS`
- Timezone offsets in minutes
- Pagination: `pageIndex` (0-based), `pageSize`
- Sorting: `sortField`, `sortType` (asc/desc)

## API Rate Limiting
Unknown - not documented in this reverse engineering effort. Use responsibly.

## CORS & Security
The API appears to require proper origin headers and authentication tokens. Cross-origin requests from unauthorized domains will likely be blocked.

## Next Steps for Full API Documentation

To complete this API documentation, you would need to:

1. **Capture Request/Response Bodies**: Use browser DevTools or a proxy to capture actual request payloads and response structures
2. **Document Authentication**: Determine token format (JWT?) and header requirements
3. **Identify Required vs Optional Parameters**: Test each endpoint to determine parameter requirements
4. **Error Response Format**: Document error codes and messages
5. **Rate Limits**: Test to discover any rate limiting policies
6. **WebSocket Endpoints**: Check if live translation uses WebSocket for real-time updates
7. **File Upload Format**: Document multipart/form-data structure for audio uploads
8. **Device Protocol**: Investigate how the HiDoc P1 device communicates (USB protocol, file transfer format)

## Legal Notice

This is an unofficial API documentation created through reverse engineering for personal and educational purposes. HiNotes and HiDoc are trademarks of their respective owners. Use of this API may violate the service's Terms of Service. Always respect the official terms and conditions when interacting with any web service.
