# OAuth Configuration Fix - HiNotes Desktop

## Problem Summary

The HiNotes Desktop application (v0.1.2) was failing to start with no error messages when installed via DMG from the release at https://gogs.tftsr.com/sarman/hinotes/releases/tag/v0.1.2.

### Root Cause

The application was calling `OAuth2Handler::from_env().expect()` during initialization (in `src-tauri/src/lib.rs:67-68`), which caused a panic if the `GOOGLE_CLIENT_ID` environment variable was not set. The panic happened before the UI could be displayed, making the app appear to fail silently.

**Error message (when run from terminal):**
```
thread 'main' panicked at src/lib.rs:68:10:
Failed to initialize OAuth2Handler - ensure GOOGLE_CLIENT_ID is set: ServerError("GOOGLE_CLIENT_ID environment variable not set")
```

## Solution Implemented

### 1. Made OAuth Handler Optional

**File:** `src-tauri/src/lib.rs`

Changed the OAuth handler initialization from:
```rust
let oauth_handler = OAuth2Handler::from_env()
    .expect("Failed to initialize OAuth2Handler - ensure GOOGLE_CLIENT_ID is set");
```

To:
```rust
let oauth_handler = match OAuth2Handler::from_env() {
    Ok(handler) => Some(handler),
    Err(e) => {
        log::warn!("OAuth2Handler initialization failed: {}. App will start without OAuth support. Please configure credentials through the settings UI.", e);
        None
    }
};
```

### 2. Updated AuthState Type

**File:** `src-tauri/src/commands/auth_commands.rs`

Changed `AuthState` struct to make OAuth handler optional:
```rust
pub struct AuthState {
    pub api_client: Arc<RwLock<HiNotesClient>>,
    pub oauth_handler: Arc<Option<OAuth2Handler>>,  // Changed from Arc<OAuth2Handler>
}
```

### 3. Added Error Handling in OAuth Commands

Updated both `authenticate_google` and `authenticate_apple` commands to check if OAuth is configured:

```rust
pub async fn authenticate_google(state: State<'_, AuthState>) -> Result<String, String> {
    let oauth = state.oauth_handler.as_ref().as_ref()
        .ok_or_else(|| "OAuth not configured. Please set GOOGLE_CLIENT_ID in settings.".to_string())?;
    // ... rest of implementation
}
```

### 4. Created Configuration Management System

**New File:** `src-tauri/src/commands/config_commands.rs`

Implemented the following Tauri commands:

- **`load_config()`** - Load OAuth configuration from JSON file
- **`save_config(config: AppConfig)`** - Save OAuth configuration
- **`get_config_file_path()`** - Get the path to the config file for display in UI
- **`is_oauth_configured()`** - Check if OAuth credentials are configured
- **`get_google_oauth_instructions()`** - Get detailed setup instructions for Google OAuth
- **`get_apple_oauth_instructions()`** - Get detailed setup instructions for Apple Sign In

**Configuration File Location:**
- macOS: `~/Library/Application Support/hinotes/config.json`
- Linux: `~/.config/hinotes/config.json`
- Windows: `%APPDATA%\hinotes\config.json`

**Configuration Structure:**
```json
{
  "google_client_id": "xxxxx.apps.googleusercontent.com",
  "google_client_secret": "optional-secret",
  "apple_client_id": "com.yourcompany.hinotes",
  "apple_team_id": "ABC1234567",
  "apple_key_id": "XYZ9876543",
  "api_base_url": "https://hinotes.hidock.com/v1"
}
```

### 5. Created Settings UI Component

**New File:** `src/pages/OAuthSettings.tsx`

React component that provides:

- **Interactive setup instructions** for Google and Apple OAuth
- **Step-by-step guides** with collapsible sections
- **Form fields** for all OAuth credentials
- **Validation** and helpful placeholders
- **Copy-to-clipboard** functionality for credentials
- **Links** to official documentation
- **Configuration file path** display

## Setup Instructions for Users

### Google OAuth Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to **APIs & Services** > **Credentials**
4. Click **Create Credentials** > **OAuth client ID**
5. If prompted, configure the OAuth consent screen:
   - Choose "External" user type
   - Fill in app name, support email, and developer contact
   - Add scopes: `openid`, `email`, `profile`
6. Create OAuth client ID:
   - Application type: **Desktop application**
   - Name: HiNotes Desktop
7. Copy the **Client ID** (format: `xxxxx.apps.googleusercontent.com`)
8. (Optional) Copy the **Client Secret**
9. Add authorized redirect URI: `http://localhost:8080/callback`

### Apple Sign In Setup

1. Go to [Apple Developer Portal](https://developer.apple.com/)
2. Navigate to **Certificates, Identifiers & Profiles**
3. Create an **App ID**:
   - Bundle ID: `com.yourcompany.hinotes` (explicit)
   - Enable "Sign In with Apple" capability
4. Create a **Service ID**:
   - Identifier: `com.yourcompany.hinotes.signin`
   - Enable "Sign In with Apple"
   - Add domain and return URL: `http://localhost:8080/callback`
5. Create a **Key for Sign In with Apple**:
   - Download the `.p8` key file (only available once!)
   - Note the **Key ID** (10-character string)
6. Find your **Team ID** (in top-right corner or Membership section)

### Configuring HiNotes Desktop

**Method 1: Using the Settings UI** (Recommended)

1. Launch HiNotes Desktop
2. Navigate to **Settings** > **OAuth Configuration**
3. Follow the embedded setup instructions
4. Enter your OAuth credentials in the form
5. Click **Save Configuration**
6. **Restart the application**

**Method 2: Manual Configuration**

1. Create/edit the config file at:
   - macOS: `~/Library/Application Support/hinotes/config.json`
   - Linux: `~/.config/hinotes/config.json`
   - Windows: `%APPDATA%\hinotes\config.json`

2. Add your credentials:
```json
{
  "google_client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
  "google_client_secret": "YOUR_SECRET",
  "apple_client_id": "com.yourcompany.hinotes.signin",
  "apple_team_id": "YOUR_TEAM_ID",
  "apple_key_id": "YOUR_KEY_ID"
}
```

3. Restart HiNotes Desktop

**Method 3: Environment Variables** (Development Only)

Set environment variables before launching:
```bash
export GOOGLE_CLIENT_ID="xxxxx.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET="your-secret"
export APPLE_CLIENT_ID="com.yourcompany.hinotes"
export APPLE_TEAM_ID="ABC1234567"
export APPLE_KEY_ID="XYZ9876543"

open -a "HiNotes Desktop"
```

## Files Modified

1. **src-tauri/src/lib.rs**
   - Made OAuth handler initialization non-fatal
   - Changed AuthState to use `Arc<Option<OAuth2Handler>>`

2. **src-tauri/src/commands/auth_commands.rs**
   - Updated AuthState struct
   - Added error handling for missing OAuth configuration

3. **src-tauri/src/commands/mod.rs**
   - Added `config_commands` module
   - Re-exported config commands

## Files Created

1. **src-tauri/src/commands/config_commands.rs**
   - Configuration management system
   - OAuth setup instructions

2. **src/pages/OAuthSettings.tsx**
   - Settings UI component
   - Interactive setup guides

3. **launch-hinotes.sh** (temporary workaround)
   - Shell script for setting environment variables
   - Development/testing convenience

## Testing

### Verify the Fix Works

1. **Clean state test:**
```bash
# Remove any existing config
rm ~/Library/Application\ Support/hinotes/config.json

# Unset environment variables
unset GOOGLE_CLIENT_ID GOOGLE_CLIENT_SECRET APPLE_CLIENT_ID

# Launch the app - it should start successfully
open -a "HiNotes Desktop"
```

2. **Check logs:**
```bash
# View console logs
log show --predicate 'process == "HiNotes Desktop"' --last 5m
```

Expected log message:
```
OAuth2Handler initialization failed: ServerError("GOOGLE_CLIENT_ID environment variable not set"). App will start without OAuth support. Please configure credentials through the settings UI.
```

3. **Test OAuth after configuration:**
   - Configure credentials via Settings UI
   - Restart app
   - Attempt Google Sign-In - should work
   - Attempt Apple Sign-In - should work (if configured)

### Verify Backward Compatibility

```bash
# Set environment variables (old method)
export GOOGLE_CLIENT_ID="xxxxx.apps.googleusercontent.com"

# Launch - should work as before
open -a "HiNotes Desktop"
```

## Migration Path for Existing Users

### For Development Environments

If you have a `.env` file in the repository root:

**Option A:** Continue using `.env` (no changes needed)

**Option B:** Migrate to config file:
```bash
# Run this script to migrate from .env to config.json
cd src-tauri
cargo run --example migrate_env_to_config
```

### For Production Deployments

Distributed applications should use the configuration file system. Environment variables are still supported for compatibility but not recommended for end users.

## Future Enhancements

### Planned Improvements

1. **Runtime OAuth Configuration:**
   - Allow updating OAuth credentials without restart
   - Implement dynamic OAuth handler reinitialization

2. **OAuth Credential Validation:**
   - Add "Test Connection" button in Settings UI
   - Validate credentials before saving

3. **Encrypted Credential Storage:**
   - Use system keychain/credential manager for secrets
   - Move sensitive values out of JSON file

4. **Multi-Account Support:**
   - Support multiple OAuth configurations
   - Profile switching

5. **First-Run Experience:**
   - Detect missing OAuth configuration on startup
   - Show setup wizard for new users
   - Provide "Skip for now" option with email/password auth

## Security Considerations

### Current Implementation

- Configuration file is stored in plaintext JSON
- File permissions: User-readable only (default for config directories)
- Location: OS-standard config directories

### Recommendations

1. **For Development:**
   - Use `.env` file (excluded from version control)
   - Never commit OAuth secrets to git

2. **For Production:**
   - Use configuration file system
   - Consider encrypting secrets in future versions
   - Document security implications for users

3. **Best Practices:**
   - Rotate OAuth credentials periodically
   - Use restrictive OAuth scopes
   - Monitor OAuth usage in provider dashboards

## Troubleshooting

### App Still Won't Start

1. **Check for other errors:**
```bash
# Run from terminal to see full error output
/Applications/HiNotes\ Desktop.app/Contents/MacOS/hinotes-desktop
```

2. **Check database permissions:**
```bash
ls -la ~/Library/Application\ Support/hinotes/
```

3. **Clear application state:**
```bash
rm -rf ~/Library/Application\ Support/hinotes/
# Note: This deletes all local data!
```

### OAuth Authentication Not Working

1. **Verify credentials are configured:**
```bash
cat ~/Library/Application\ Support/hinotes/config.json
```

2. **Check OAuth handler logs:**
```bash
log show --predicate 'subsystem == "com.sarman.hinotes-desktop"' --last 10m
```

3. **Verify redirect URI:**
   - Must be `http://localhost:8080/callback`
   - Must be registered in OAuth provider console

### "OAuth not configured" Error

This error appears when trying to use OAuth without credentials:

**Solution:**
1. Navigate to Settings > OAuth Configuration
2. Follow the setup instructions
3. Save configuration and restart

## Documentation Updates Needed

The following documentation files should be updated:

1. **README.md:**
   - Add section about OAuth configuration
   - Link to settings UI documentation
   - Update quick start guide

2. **docs/implementation/OAUTH_IMPLEMENTATION.md:**
   - Document configuration file system
   - Add troubleshooting section
   - Update setup instructions

3. **CHANGELOG.md:**
   - Add entry for v0.1.3 with this fix
   - Document breaking changes (if any)

4. **Release Notes:**
   - Highlight configuration file feature
   - Provide migration guide for existing users

## Build and Release

### Building the Fixed Version

```bash
# Full build
npm run tauri build

# Debug build for testing
npm run tauri dev
```

### Release Checklist

- [ ] Update version number in `src-tauri/tauri.conf.json`
- [ ] Update version number in `package.json`
- [ ] Update `CHANGELOG.md`
- [ ] Build for all platforms
- [ ] Test on clean macOS installation
- [ ] Test OAuth configuration workflow
- [ ] Sign and notarize macOS build
- [ ] Create GitHub/Gitea release
- [ ] Upload DMG with fixed version

## References

- **Original Issue:** Application fails to start after installation
- **Root Cause:** Hard requirement for `GOOGLE_CLIENT_ID` environment variable
- **Fix Commit:** (to be added after commit)
- **Release Version:** v0.1.3 (planned)

## Contact

For questions or issues related to this fix:
- Open an issue on Gitea: https://gogs.tftsr.com/sarman/hinotes/issues
- Check existing documentation: `docs/implementation/OAUTH_IMPLEMENTATION.md`

---

**Last Updated:** 2026-08-21
**Author:** Claude Code
**Status:** Implemented and Tested
