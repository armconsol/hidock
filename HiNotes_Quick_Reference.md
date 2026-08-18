# HiNotes API - Quick Reference Guide

## 🔗 Base URL
```
https://hinotes.hidock.com/v1
```

## 🔑 Authentication
All authenticated requests require a Bearer token in the Authorization header:
```
Authorization: Bearer YOUR_TOKEN_HERE
```

## 📋 Most Common Endpoints

### 🔐 Authentication
```bash
# Sign In
POST /oauth2/signin/google
POST /oauth2/signin/apple
POST /user/signin
  { "email": "user@example.com", "password": "secret" }

# Register
POST /user/register
  { "email": "user@example.com", "password": "secret", "name": "John Doe" }
```

### 👤 User Info
```bash
# Get current user
POST /user/info

# Update name
POST /user/rename
  { "name": "New Name" }
```

### 📱 Devices
```bash
# List devices
POST /user/device/list

# Bind device
POST /user/device/bind
  { "deviceId": "ABC123", "deviceName": "My HiDoc P1" }

# Check status
GET /user/device/status
```

### 📝 Notes
```bash
# List notes (GET with query params)
GET /note/recording/list?folderId=-1&pageIndex=0&pageSize=20&sortType=desc&sortField=createtime

# List whispers
GET /note/whisper/list?pageSize=20&sortField=create_time

# Delete note
POST /note/delete
  { "noteId": "n_123456" }

# Rate transcription
POST /note/rate
  { "noteId": "n_123456", "rating": 5 }
```

### 📁 Folders
```bash
# List folders
POST /folder/list

# Create folder
POST /folder/create
  { "name": "My Folder" }

# Rename folder
POST /folder/rename
  { "folderId": "f_123", "name": "New Name" }

# Delete folder
POST /folder/remove
  { "folderId": "f_123" }
```

### ✅ To-Dos
```bash
# List todos (GET with query params)
GET /todo/list?pageSize=10&state=open&tzOffset=300&dueDateStart=2026-08-18 00:00:00&dueDateEnd=2026-08-19 00:00:00

# Update description
POST /todo/update/description
  { "todoId": "t_123", "description": "Updated task" }

# Delete todo
POST /todo/delete
  { "todoId": "t_123" }
```

### 📅 Calendar
```bash
# List events (GET with query params)
GET /calendar/event/list?start_time=2026-08-01 00:00:00&end_time=2026-08-31 23:59:59&tz_offset=300

# Add event
POST /calendar/event/add
  { "title": "Meeting", "startTime": "2026-08-18 10:00:00", "endTime": "2026-08-18 11:00:00" }
```

### 🔄 Sync
```bash
# Sync changes
POST /changes

# Get entry info
POST /entry/info
```

## 🐍 Python Example
```python
import requests

BASE_URL = "https://hinotes.hidock.com/v1"

# Sign in
response = requests.post(f"{BASE_URL}/user/signin", json={
    "email": "user@example.com",
    "password": "password"
})
token = response.json()['token']

# Set up authenticated session
headers = {"Authorization": f"Bearer {token}"}

# Get user info
user = requests.post(f"{BASE_URL}/user/info", headers=headers).json()

# List notes
notes = requests.get(
    f"{BASE_URL}/note/recording/list",
    params={"pageSize": 20, "folderId": "-1"},
    headers=headers
).json()

# List devices
devices = requests.post(f"{BASE_URL}/user/device/list", headers=headers).json()
```

## 💻 curl Examples

### Sign In
```bash
curl -X POST https://hinotes.hidock.com/v1/user/signin \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret"}'
```

### Get User Info
```bash
curl -X POST https://hinotes.hidock.com/v1/user/info \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json"
```

### List Notes
```bash
curl -X GET "https://hinotes.hidock.com/v1/note/recording/list?pageSize=20&folderId=-1&pageIndex=0&sortType=desc&sortField=createtime" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### List Devices
```bash
curl -X POST https://hinotes.hidock.com/v1/user/device/list \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json"
```

### Create Folder
```bash
curl -X POST https://hinotes.hidock.com/v1/folder/create \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"My New Folder"}'
```

### Add To-Do
```bash
curl -X POST https://hinotes.hidock.com/v1/todo/update/description \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"todoId":"t_123","description":"New task description"}'
```

## 🔧 Common Parameters

### Pagination
```
pageIndex=0       # 0-based page number
pageSize=20       # Items per page
```

### Sorting
```
sortField=createtime    # Field to sort by
sortType=desc          # 'asc' or 'desc'
```

### Date/Time Format
```
2026-08-18 14:30:00    # YYYY-MM-DD HH:MM:SS
```

### Timezone
```
tz_offset=300          # Offset in minutes (e.g., 300 = UTC-5)
tzOffset=300           # Some endpoints use camelCase
```

### Folder ID
```
folderId=-1           # -1 means "all folders"
folderId=f_123        # Specific folder
```

## 📊 Data Types

### Note Object
```json
{
  "id": "n_6156623080614670336",
  "title": "2026-08-17 Meeting Notes",
  "content": "Transcription text...",
  "folderId": "f_123",
  "createTime": "2026-08-17 16:37:00",
  "duration": "37:10"
}
```

### Device Object
```json
{
  "id": "d_123456",
  "name": "My HiDoc P1",
  "status": "connected"
}
```

### Todo Object
```json
{
  "id": "t_123456",
  "description": "Task description",
  "dueDate": "2026-08-18 19:00:00",
  "state": "open",
  "smartLabel": "work"
}
```

## ⚠️ Important Notes

### HTTP Methods
- Most endpoints use **POST** even for reads
- Only some list endpoints use **GET**
- Check documentation for each endpoint

### Authentication
- Token format unknown (likely JWT)
- Token obtained during sign-in
- Include in Authorization header as Bearer token

### Rate Limiting
- Rate limits unknown
- Use responsibly
- Implement exponential backoff

### Error Handling
- Error response format unknown
- Always check HTTP status codes
- Implement try/catch blocks

## 🚨 Warnings

1. **Unofficial API** - Not officially documented or supported
2. **May violate ToS** - Using this API may breach HiNotes Terms of Service
3. **Subject to change** - Endpoints can change without notice
4. **No SLA** - No uptime or performance guarantees
5. **Legal risk** - Use at your own risk

## 💡 Best Practices

1. **Cache data** - Don't poll unnecessarily
2. **Batch requests** - Group related operations
3. **Handle errors** - Implement retry logic
4. **Respect limits** - Add delays between requests
5. **Secure tokens** - Never commit tokens to version control
6. **Test thoroughly** - Start with read-only operations

## 📚 Additional Resources

- **Full Documentation**: See `HiNotes_API_Documentation.md`
- **OpenAPI Spec**: See `HiNotes_OpenAPI.yaml`
- **Python Client**: See `hinotes_client.py`
- **Summary**: See `HiNotes_API_Summary.md`

## 🔍 Need More Info?

1. Use browser DevTools Network tab
2. Inspect actual requests/responses
3. Check console for error messages
4. Analyze JavaScript source code
5. Contact HiDock for official API access

---

**Last Updated**: August 18, 2026  
**Status**: Unofficial / Reverse Engineered  
**Use**: Educational purposes only
