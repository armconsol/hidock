# HiNotes API — Verified Endpoint Map (Unofficial)

> Reverse-engineered + LIVE-VERIFIED against `https://hinotes.hidock.com/v1` on 2026-09-02.
> Supersedes assumptions in `HiNotes_API_Documentation.md` / `HiNotes_OpenAPI.yaml` where they conflict — those files were written from static JS scanning only and had several wrong assumptions (see "Corrections" below). Use `hinotes_client.py` (v2) as the source of truth; this doc explains why it's shaped the way it is.

## Auth model (important — differs from most REST APIs)

- **No cookies.** The web app is an SPA; auth state lives in `localStorage['accessToken']`.
- **Custom header**, not standard Bearer: every authenticated request sends
  `accesstoken: <token>` (all lowercase, no "Authorization:" or "Bearer" prefix).
- Token is an **opaque random string** (64 chars observed), not a JWT — no
  client-decodable expiry. Unknown fixed TTL; detect death via 401/403 on
  any call, or `client.is_token_valid()`.
- **Getting a token without solving a captcha isn't possible.** `/v1/user/signin`
  requires a real Google reCAPTCHA v2 response token and the server checks it
  (`{"error":10002,"message":"captcha_required"}` if missing/invalid). The
  practical path: log in once in a real browser, run
  `localStorage.getItem('accessToken')` in DevTools, and feed that string to
  the client. Repeat on expiry.
- Google/Apple OAuth sign-in (`/v1/oauth2/signin/google|apple`) exists but
  routes through the OAuth provider's own bot defenses — not easier to
  automate than the captcha, and doesn't yield a durable API key either.

## Request format (important — differs from most JSON APIs)

- Most POST endpoints are sent as **`multipart/form-data`** (the app builds
  a `FormData` and does NOT set `Content-Type: application/json`). Sending
  raw JSON bodies gets a generic `400 Bad Request` with no useful message.
- GET endpoints take standard query params.
- Response envelope is normally JSON: `{"error": 0, "message": "success", "data": {...}}`.
  A nonzero `error` with a message key indicates a specific failure
  (`captcha_required`, `sys_failure`, etc.) — codes are not otherwise documented.
- A few endpoints (e.g. `/note/recording/detail`, `/note/recording/transcriptions`)
  return content that looks XML-ish (`<Result><error>0</error>...`) but is
  transparently handled by `requests.json()` / browsers as JSON-equivalent —
  don't be alarmed by the angle brackets in raw curl output.

## Corrections vs. the original reverse-engineering pass

| Area | Old assumption (wrong) | Verified reality |
|---|---|---|
| Sign-in body | JSON `{email,password}` | multipart form + required `captcha` field |
| Auth header | `Authorization: Bearer <token>` | `accesstoken: <token>` |
| `folderId=-1` | "all folders" | Returns **empty**. Omit the param entirely for all notes. |
| Note detail | `/v1/note/{id}/detail` | `/v1/note/recording/detail?noteId={id}` (or `/v1/note/whisper/detail?noteId=` for whispers) — the bundle's template literal's first segment is the note **type**, not the id. |
| Transcript | Not documented | `GET /v1/note/recording/transcriptions?noteId={id}` — returns array of per-sentence segments with `speaker`, `beginTime`/`endTime` (ms), `sentence`. |

## Verified working endpoints (called live, 200 + real data)

```
POST /v1/user/info                                  -> user profile, plan, totalNoteCount
POST /v1/user/device/list                           -> paired HiDoc devices
POST /v1/folder/list                                -> folders + noteCount per folder
GET  /v1/note/recording/list?pageIndex=&pageSize=&sortType=&sortField=
                                                      -> paginated notes, includes conciseSummary
GET  /v1/note/recording/detail?noteId=               -> single note metadata
GET  /v1/note/recording/transcriptions?noteId=       -> full per-sentence transcript, speaker-labelled
GET  /v1/note/whisper/list?pageSize=&sortField=      -> quick voice notes (whispers)
GET  /v1/todo/list?...                               -> todos (unchanged from original docs)
GET  /v1/calendar/event/list?...                     -> calendar events (unchanged)
POST /v1/entry/info                                  -> app entry payload (motivational quote, flags)
POST /v1/changes                                     -> sync/changes feed
```

## Endpoints discovered in bundle but NOT yet live-tested

See `HiNotes_API_Documentation.md` for the full original list (90+). Notably
still unverified: audio file download URL (no `audioUrl`/`downloadUrl` field
found in note detail responses — `hasAudio: true` exists but the actual
retrieval path is unconfirmed, likely tied to device file transfer
`/v1/user/device/file/get`), sharing endpoints, template CRUD, subscription
receipts, referral program.

## Recommended next steps for the full mapping the user wants

1. **Audio download**: capture a real "download recording" click in the web
   UI via Playwright/browser network capture — the JS bundle only shows
   `hasAudio: true` on note detail, no plain URL field visible in this pass.
2. **Whisper detail equivalent**: confirm `/v1/note/whisper/detail?noteId=` and
   `/v1/note/whisper/transcriptions?noteId=` symmetry with recording notes.
3. **Error code table**: build up `{error: N}` -> meaning as they're hit
   (`10002` = captcha_required, `90000` = sys_failure so far).
4. **Rate limits**: none observed yet; still unknown — throttle client usage.
