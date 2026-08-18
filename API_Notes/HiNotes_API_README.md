# HiNotes API - Unofficial Documentation

> **⚠️ DISCLAIMER**: This is unofficial API documentation created through reverse engineering. Using this API may violate HiNotes Terms of Service. Always contact HiDock for official API access before production use.

## 📖 Overview

This repository contains comprehensive documentation for the HiNotes web application API, used with the **HiDoc P1** USB audio transcription device. The documentation was created by analyzing network traffic and JavaScript source code from the HiNotes web application.

## 🎯 What is HiNotes?

HiNotes is a cloud-based note-taking and transcription service designed for use with the HiDoc P1 hardware device. It provides:

- 🎙️ Audio recording and transcription
- 🗣️ Multi-speaker identification
- 🌐 Live translation
- 📝 Note organization and management
- ✅ To-do list integration
- 📅 Calendar synchronization
- 🔄 Cross-device sync

## 📁 Documentation Files

| File | Description |
|------|-------------|
| **HiNotes_API_Documentation.md** | Complete API reference with 90+ endpoints |
| **HiNotes_OpenAPI.yaml** | OpenAPI 3.0 specification for API tools |
| **HiNotes_Quick_Reference.md** | Quick lookup guide with curl examples |
| **HiNotes_API_Summary.md** | Executive summary and methodology |
| **hinotes_client.py** | Python client library (starter template) |
| **HiNotes_API_README.md** | This file |

## 🚀 Quick Start

### Using curl

```bash
# 1. Sign in
TOKEN=$(curl -s -X POST https://hinotes.hidock.com/v1/user/signin \
  -H "Content-Type: application/json" \
  -d '{"email":"your@email.com","password":"yourpassword"}' \
  | jq -r '.token')

# 2. Get user info
curl -X POST https://hinotes.hidock.com/v1/user/info \
  -H "Authorization: Bearer $TOKEN"

# 3. List notes
curl -X GET "https://hinotes.hidock.com/v1/note/recording/list?pageSize=20&folderId=-1" \
  -H "Authorization: Bearer $TOKEN"
```

### Using Python

```python
from hinotes_client import HiNotesClient

# Initialize and authenticate
client = HiNotesClient()
client.authenticate_with_credentials("your@email.com", "yourpassword")

# Get user info
user = client.get_user_info()

# List notes
notes = client.list_notes(page_size=20)

# List devices
devices = client.list_devices()
```

### Using the OpenAPI Spec

Import `HiNotes_OpenAPI.yaml` into:
- **Postman**: Generate requests automatically
- **Swagger UI**: Interactive API documentation
- **OpenAPI Generator**: Generate client libraries in any language

## 📊 API Statistics

- **Base URL**: `https://hinotes.hidock.com/v1`
- **Total Endpoints**: 90+
- **Categories**: 15
- **Authentication**: OAuth2 (Google, Apple) + Email/Password
- **Format**: JSON
- **Protocol**: HTTPS

### Endpoint Categories

| Category | Endpoints | Description |
|----------|-----------|-------------|
| Authentication | 4 | OAuth2, sign in, register |
| User Management | 17 | Profile, settings, verification |
| Device Management | 9 | HiDoc P1 binding, file transfer |
| Notes | 12 | Recording, whisper notes, CRUD |
| Audio Operations | 3 | Merge, replace, save as new |
| Folders | 4 | Organization, CRUD |
| To-Do | 5 | Task management |
| Calendar | 4 | Event sync, integration |
| Live Translation | 3 | Real-time translation |
| Templates | 9 | Note templates |
| Smart Labels | 3 | Categories, tags |
| Vocabulary | 3 | Custom transcription dictionary |
| Settings | 4 | User preferences |
| Sharing | 2 | Share links |
| Subscription | 3 | Billing via RevenueCat |
| Referral | 7 | Rewards program |
| Sync | 2 | Data synchronization |

## 🔍 Research Methodology

### Tools Used
1. **Playwright Browser Automation** - Network traffic capture
2. **Browser DevTools** - Request/response inspection
3. **curl** - JavaScript source analysis
4. **grep** - Pattern extraction from source

### Process
1. ✅ Navigated to HiNotes web application
2. ✅ Authenticated with Google OAuth2
3. ✅ Captured network traffic during normal usage
4. ✅ Downloaded and analyzed JavaScript bundles
5. ✅ Extracted API endpoint patterns
6. ✅ Documented observed request structures
7. ✅ Created comprehensive documentation

## 🎓 Use Cases

### Personal Projects
- ✅ Backup your HiNotes data
- ✅ Build custom integrations
- ✅ Export notes to other systems
- ✅ Automate workflows

### Development
- ✅ Build alternative clients
- ✅ Create automation scripts
- ✅ Integrate with other tools
- ✅ Develop custom features

### Research
- ✅ Study API design patterns
- ✅ Understand audio transcription systems
- ✅ Learn reverse engineering techniques
- ✅ Educational purposes

## ⚠️ Legal & Ethical Considerations

### Important Warnings

1. **Unofficial Documentation**
   - Not created or endorsed by HiDock
   - No support or warranty provided
   - Subject to change without notice

2. **Terms of Service**
   - Using this API may violate HiNotes ToS
   - Review terms before using
   - Consider official API access

3. **Rate Limiting**
   - Unknown rate limits
   - Use responsibly
   - Implement backoff strategies

4. **Security**
   - Protect authentication tokens
   - Use HTTPS only
   - Follow security best practices

5. **Privacy**
   - Respect user data
   - Follow data protection laws
   - Handle transcriptions securely

### Recommendations

- 📧 **Contact HiDock** for official API access
- 📋 **Review Terms of Service** before use
- 🔒 **Implement security** properly
- ⏱️ **Respect rate limits** (when known)
- 🧪 **Test carefully** on non-production data

## 🛠️ Technical Details

### Architecture
- **Frontend**: React + Vite
- **UI Framework**: Arco Design
- **Audio**: FFmpeg WebAssembly
- **i18n**: English, Chinese, Japanese
- **State**: Custom state management

### Authentication Flow
```
User → OAuth2 Provider (Google/Apple)
     ↓
  Token Generation
     ↓
  HiNotes API (Bearer token)
     ↓
  Protected Resources
```

### Data Flow
```
HiDoc P1 Device → USB → Browser
                         ↓
                    HiNotes Web App
                         ↓
                    API (HTTPS)
                         ↓
                  Cloud Storage
```

## 📚 Additional Resources

### Official Links
- **HiNotes Web**: https://hinotes.hidock.com
- **HiDock Website**: https://hidock.com (assumed)

### Third-Party Services
- **RevenueCat**: Subscription management
- **Google APIs**: Calendar, Drive, OAuth
- **Apple ID**: Authentication

### Related Tools
- **Postman**: API testing
- **Swagger UI**: Interactive docs
- **OpenAPI Generator**: Client generation
- **Charles Proxy**: Traffic inspection

## 🤝 Contributing

Found errors or have additional information?

1. Document your findings
2. Test thoroughly
3. Verify accuracy
4. Submit corrections

**Note**: This is community documentation. Contributions should be accurate and well-tested.

## 📧 Contact

For official API access and support:
- Contact HiDock/HiNotes directly
- Visit official website
- Use official support channels

For documentation questions:
- Open an issue
- Submit corrections
- Share findings

## 📜 License

**Educational Use Only**

This documentation is provided for educational and research purposes. No license is granted for commercial use. Always respect intellectual property rights and terms of service.

## 🙏 Acknowledgments

- **HiDock Team**: For creating HiNotes and HiDoc P1
- **Open Source Community**: For reverse engineering tools
- **Claude Code (Anthropic)**: For documentation generation

## 🔄 Updates

| Date | Version | Changes |
|------|---------|---------|
| 2026-08-18 | 1.0.0 | Initial release |

## 📝 Notes

### What's Documented
- ✅ 90+ API endpoints discovered
- ✅ Request patterns observed
- ✅ URL structures analyzed
- ✅ Query parameters documented

### What's Missing
- ❌ Complete request/response bodies
- ❌ Error response formats
- ❌ Authentication token format
- ❌ Rate limiting policies
- ❌ WebSocket protocols (if any)
- ❌ USB device communication protocol

### Next Steps
1. Capture actual request/response bodies
2. Document authentication token structure
3. Test rate limiting behavior
4. Map error codes and messages
5. Investigate WebSocket usage
6. Study HiDoc P1 USB protocol

## 🎯 Goals

This documentation aims to:
- 📖 Provide comprehensive API reference
- 🔧 Enable custom integrations
- 🎓 Support educational research
- 🤝 Foster community contributions
- ⚠️ Raise awareness of ToS implications

## ⚖️ Final Warning

**USE AT YOUR OWN RISK**

This is reverse-engineered documentation. Using these APIs in production:
- May violate Terms of Service
- May result in account termination
- Has no official support
- Has no uptime guarantees
- May change without notice

**Always seek official API access from HiDock before production use.**

---

**Created**: August 18, 2026  
**Method**: Reverse Engineering via Network Analysis  
**Tool**: Claude Code (Anthropic)  
**Status**: Unofficial Community Documentation  
**Version**: 1.0.0
