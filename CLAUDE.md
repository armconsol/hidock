# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains **reverse-engineered API documentation** for the **HiNotes** web application, which is used with the **HiDoc P1 USB audio transcription device**. The documentation was created by analyzing network traffic and JavaScript source code.

**⚠️ CRITICAL**: This is unofficial documentation created through reverse engineering. Using this API may violate HiNotes Terms of Service. Always include appropriate disclaimers when working with this content.

## Repository Structure

### Documentation Files

- **HiNotes_API_Documentation.md** - Comprehensive reference with 90+ endpoints organized by category
- **HiNotes_OpenAPI.yaml** - OpenAPI 3.0 specification for tooling integration (Postman, Swagger, code generators)
- **HiNotes_Quick_Reference.md** - Quick lookup guide with curl examples
- **HiNotes_API_Summary.md** - Executive summary and research methodology
- **HiNotes_API_README.md** - Main documentation entry point

### Code Files

- **hinotes_client.py** - Python client library (starter template with core endpoints implemented)

### Research Artifacts

- **hinotes_authenticated_requests.txt** - Captured authenticated API calls
- **hinotes_network_requests.txt** - Initial network traffic capture
- **hinotes_note_detail_requests.txt** - Detailed note-related requests
- **hinotes_snapshot.md** - Browser DOM snapshot of HiNotes web interface
- **hinotes_home.png**, **hinotes_page.png** - Screenshots of HiNotes web application

## API Architecture

### Base URL
```
https://hinotes.hidock.com/v1
```

### Authentication
- **OAuth2**: Google and Apple sign-in
- **Email/Password**: Direct authentication
- **Token Format**: Bearer tokens in `Authorization` header

### Endpoint Categories (90+ total)

1. **Authentication** (4) - OAuth2, sign in, register
2. **User Management** (17) - Profile, settings, verification
3. **Device Management** (9) - HiDoc P1 binding, file transfer
4. **Notes** (12) - Recording notes, whisper notes, CRUD operations
5. **Audio Operations** (3) - Merge, replace, save as new
6. **Folders** (4) - Organization, CRUD
7. **To-Do** (5) - Task management
8. **Calendar** (4) - Event sync, integration
9. **Live Translation** (3) - Real-time translation
10. **Templates** (9) - Note templates
11. **Smart Labels** (3) - Categories, tags
12. **Vocabulary** (3) - Custom transcription dictionary
13. **Settings** (4) - User preferences
14. **Sharing** (2) - Share links
15. **Subscription** (3) - Billing via RevenueCat
16. **Referral** (7) - Rewards program
17. **Sync** (2) - Data synchronization

## Working with the Python Client

### Installation
No external dependencies required beyond `requests`:
```bash
pip install requests
```

### Usage Pattern
```python
from hinotes_client import HiNotesClient

# Initialize and authenticate
client = HiNotesClient()
client.authenticate_with_credentials("email@example.com", "password")

# Use methods like:
# client.get_user_info()
# client.list_notes()
# client.list_devices()
# client.list_todos()
```

### Extending the Client

When adding new endpoints to `hinotes_client.py`:

1. **Follow existing patterns** - Use `_request()` method for all API calls
2. **Add type hints** - Use `typing` module for parameters and return types
3. **Document parameters** - Include clear docstrings with Args and Returns sections
4. **Handle dates consistently** - Use `datetime` objects, format as `YYYY-MM-DD HH:MM:SS`
5. **Implement pagination** - Use `pageIndex` (0-based) and `pageSize` parameters
6. **Consider timezone** - Add `tzOffset` parameter where relevant (minutes from UTC)

## Working with Documentation

### Adding New Endpoints

When documenting newly discovered endpoints:

1. **In HiNotes_API_Documentation.md**:
   - Add under the appropriate category
   - Include HTTP method, endpoint path, description
   - Note if observed in network traffic
   - Document query parameters and expected request/response structure if known

2. **In HiNotes_OpenAPI.yaml**:
   - Add to appropriate tag section
   - Define request/response schemas under `components/schemas` if known
   - Use `bearerAuth` security scheme
   - Include description with disclaimer about unofficial status

3. **In HiNotes_Quick_Reference.md**:
   - Add curl example with placeholders for sensitive data
   - Show practical usage pattern

4. **In hinotes_client.py**:
   - Add method following existing conventions
   - Use appropriate HTTP verb via `_request()`
   - Add to `__main__` example section (commented out)

### Disclaimer Requirements

**ALL documentation files MUST include prominent disclaimers** about:
- Unofficial/reverse-engineered nature
- Potential Terms of Service violations
- "Use at your own risk" warning
- Recommendation to contact HiDock for official API access
- Educational/research purposes only

## Research Methodology

This API was discovered using:

1. **Playwright Browser Automation** - Captured network traffic while using HiNotes web app
2. **Browser DevTools** - Request/response inspection
3. **curl** - Downloaded and analyzed JavaScript source bundles
4. **grep** - Extracted API endpoint patterns from minified code

### Known Limitations

The following information is **NOT yet documented**:
- Complete request/response body structures (partially missing)
- Error response formats and error codes
- Authentication token internal structure
- Rate limiting policies
- WebSocket protocols (if any exist)
- USB device communication protocol between HiDoc P1 and browser

## Development Guidelines

### Testing API Endpoints

When testing endpoints:

1. **Start with read-only operations** - GET requests, user info
2. **Use non-production data** - Don't test with important notes/recordings
3. **Implement rate limiting** - Add delays between requests (unknown limits)
4. **Log everything** - Capture requests/responses for documentation
5. **Handle errors gracefully** - Implement retry logic with exponential backoff

### Security Practices

- Never commit real authentication tokens
- Use environment variables for credentials in examples
- Validate HTTPS is used for all requests
- Warn users about storing transcriptions securely (may contain sensitive content)

### Code Style

- Follow PEP 8 for Python code
- Use descriptive variable names (avoid abbreviations except common ones like `tz` for timezone)
- Keep methods focused on single operations
- Comment non-obvious behavior or API quirks

## Common Tasks

### Testing Authentication Flow
```bash
# Get token
curl -X POST https://hinotes.hidock.com/v1/user/signin \
  -H "Content-Type: application/json" \
  -d '{"email":"EMAIL","password":"PASS"}' | jq -r '.token'

# Use token
curl -X POST https://hinotes.hidock.com/v1/user/info \
  -H "Authorization: Bearer $TOKEN"
```

### Validating OpenAPI Spec
```bash
# Using openapi-generator-cli
docker run --rm -v "${PWD}:/local" openapitools/openapi-generator-cli validate \
  -i /local/HiNotes_OpenAPI.yaml
```

### Generating Client Libraries from OpenAPI
```bash
# Generate Python client
openapi-generator generate -i HiNotes_OpenAPI.yaml -g python -o ./generated/python

# Generate TypeScript client  
openapi-generator generate -i HiNotes_OpenAPI.yaml -g typescript-axios -o ./generated/typescript
```

## Technical Context

### HiNotes Technology Stack
- **Frontend**: React + Vite
- **UI Framework**: Arco Design
- **Audio Processing**: FFmpeg WebAssembly
- **i18n**: English, Chinese, Japanese
- **Build**: Modern JavaScript bundler (Vite)

### Third-Party Services Used
- **RevenueCat** - Subscription management
- **Google APIs** - Calendar, Drive, OAuth
- **Apple ID** - Authentication
- **PayPal** - Referral program payouts

### API Behavior Patterns
- Most endpoints use POST even for read operations
- Calendar events endpoint is polled frequently
- Real-time sync via `/v1/changes` endpoint
- Initial app state loaded via `/v1/entry/info`

## Legal & Ethical Reminders

When working on this project:

1. **Always include disclaimers** in any new documentation
2. **Never encourage ToS violations** - Recommend official API access
3. **Educational focus** - Frame work as research/learning
4. **Respect privacy** - Don't include real user data in examples
5. **No production use** - Warn against using unofficial API in production

## Contributing Updates

When updating documentation after discovering new information:

1. Verify accuracy by testing endpoints
2. Update ALL relevant files consistently (main docs, OpenAPI, quick reference, Python client)
3. Add discovery date/version notes if significant changes
4. Update the "Updates" section in HiNotes_API_README.md
5. Consider updating API Statistics section with new endpoint counts

## References

- **Official HiNotes**: https://hinotes.hidock.com
- **OpenAPI Specification**: https://swagger.io/specification/
- **RevenueCat Documentation**: https://docs.revenuecat.com/
