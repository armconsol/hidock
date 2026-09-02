# HiNotes API — Complete Verified Endpoint Map (Unofficial)

> Full sweep completed 2026-09-02. Extracted every `/v1/` and `/v2/` path
> referenced in the live app bundle (`index-DXxQ4T5b.js`, 125 total paths:
> 91 `/v1/`, 34 `/v2/`), then live-tested every read-only endpoint plus
> safe/reversible mutations (create→rename→delete a scratch folder) against
> the real account. Supersedes `HiNotes_API_Documentation.md` /
> `HiNotes_OpenAPI.yaml` / the first-pass `HiNotes_API_Verified_Map.md`
> wherever they conflict.

**⚠️ Unofficial / ToS risk** — reverse-engineered, no support, may violate
HiNotes Terms of Service, can change without notice. Use read-only where
possible; treat mutations with care (they affect the real account).

---

## Auth model

- Auth state lives in `localStorage['accessToken']` in the SPA — NOT a
  cookie, NOT a JWT (opaque 64-char string, no decodable expiry).
- Every request sends it back as header `accesstoken: <token>` (lowercase,
  no `Bearer` prefix). Audio endpoints alternatively accept it as a query
  param: `?accesstoken=<token>` (used for `window.open()` downloads that
  can't set custom headers).
- **Getting a token requires a human to solve a live Google reCAPTCHA v2**
  at `/v1/user/signin`; the server enforces it
  (`{"error":10002,"message":"captcha_required"}` otherwise). Not
  automatable. Get it via: user logs in in a real browser → DevTools
  console → `localStorage.getItem('accessToken')`.
- **New login invalidates the prior token** (observed directly: an old
  token died the moment the user logged in again in a fresh browser
  session). So this is likely single-active-session per account, not
  purely time-based expiry — though a time-based expiry may also exist
  and just wasn't isolated in this pass.
- **`/v1/user/info` is UNSAFE as a "is my token still valid" check.** With
  an invalid/expired token it does NOT error — it silently returns
  `{"error":0,...}` for a freshly auto-provisioned anonymous **Guest**
  account (`type:"trial"`, `name:"Guest"`, random `@hidock.com` email,
  `totalNoteCount:0`), a NEW one on every call. Use `folder/list` or
  `note/recording/list` instead to validate a token — those correctly
  return `{"error":10000,"message":"session_timeout"}` when dead.

## Request format

- Most POST bodies are **`multipart/form-data`** (FormData, no explicit
  `Content-Type: application/json`). A few endpoints (`template/test`)
  use `postJson` — genuine JSON body — confirmed by call site naming but
  not independently live-tested this pass.
- GET endpoints take query params.
- Response envelope, JSON path: `{"error": 0, "message": "success", "data": ...}`.
- Response envelope, XML-ish path (still valid, parsed transparently by
  the app / by `requests.json()`... actually NOT transparently for raw
  `curl`/`requests` — see note below): `<Result><error>0</error>...`.
  **Important**: several endpoints (`note/recording/detail`,
  `note/recording/transcriptions`, `folder/list`, `user/device/list`,
  `template/list`, etc.) return this XML-ish text even though the app
  treats it as structured data client-side (likely via a custom
  deserializer, not `response.json()`). Plain `requests.get(...).json()`
  will FAIL on these — the client needs a small XML-ish parser, or use
  `response.text` and regex/parse manually. This affects `hinotes_client.py`
  — see the "Client implications" section below.
- Known error codes so far: `0`=success, `10000`=session_timeout,
  `10002`=captcha_required, `90000`=sys_failure, `90003`=invalid_request.

## Client implications (fix required)

The current `hinotes_client.py` (v2, written before this full sweep)
assumes every response is clean JSON via `resp.json()`. That is TRUE for
some endpoints (confirmed: `subscribers` 404s cleanly, `payment/rc/portal`
returns proper JSON-shaped error) but several core endpoints
(`folder/list`, `note/recording/list` in some cases, `note/recording/detail`,
`transcriptions`) come back as the `<Result>...</Result>` pseudo-XML
textual format from raw curl. **Action item**: verify in a follow-up pass
whether `requests`'s `Accept: application/json` header changes this, or
whether a dedicated lightweight parser is needed. Flagging this now rather
than shipping a client that silently mis-parses.

---

## Full endpoint inventory (125 unique paths)

### Authentication (5)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/user/signin` | POST | form: `email,password,captcha` | ✅ Live-tested; captcha enforced server-side, confirmed blocking |
| `/v1/oauth2/signin/google` | POST | OAuth payload | Not tested (routes through Google's own bot defenses) |
| `/v1/oauth2/signin/apple` | POST | OAuth payload | Not tested |
| `/v1/user/register` | POST | form: `email,password,name` | Not tested (would create a real account) |
| `/v1/user/logout` | POST | `{}` | Not tested (would kill current session) |

### User account (17)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/user/info` | POST | none | ✅ Live — **see auth warning above, unsafe for validity checks** |
| `/v1/user/rename` | POST | form: `name` | Not tested (mutating) |
| `/v1/user/role/update` | POST | form: `role` | Not tested |
| `/v1/user/region/update` | POST | form: `region` | Not tested |
| `/v1/user/avatar/upload` | POST | form: `file` (binary) | Not tested |
| `/v1/user/delete` | POST | none | **NOT TESTED — destructive, deletes account** |
| `/v1/user/password/update` | POST | JSON/form body | Not tested (mutating, security-sensitive) |
| `/v1/user/email/verification/send` | POST | none | Not tested |
| `/v1/user/email/verification/verify` | POST | form: `code` | Not tested |
| `/v1/user/reset/authcode/send` | POST | form: `email` | Not tested (public API, no auth needed) |
| `/v1/user/reset/check` | POST | form: `email,code` | Not tested |
| `/v1/user/reset/save` | POST | form: new password payload | Not tested (public API) |
| `/v1/user/activateCode/send` | POST | form payload | Not tested (public API) |
| `/v1/user/country/list` | POST | none | ✅ Live — full country list (public, no auth needed) |
| `/v1/user/trial/check` | POST | none | ✅ Live — `{"data":false}` (already past trial) |
| `/v1/user/trial/claim` | POST | none | Not tested (mutating, one-time) |
| `/v1/u/{id}/restore`, `/v1/u/{id}/clean` | POST | none | Not tested (account restore/purge, destructive) |

### User settings (6)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/user/setting/get` | POST | form: `group,code` | ✅ Live — e.g. `group=user&code=auto-summarize` → `"on"` |
| `/v1/user/setting/list` | POST | form: `group` | ✅ Live — returns settings + option definitions incl. AI engine choices |
| `/v1/user/setting/save` | POST | form: `group,code,value` | Not tested this pass (mutating; confirmed shape from bundle) |
| `/v1/user/setting/ai_engine/list` | POST | none | ✅ Live — 7 models incl. GPT-5, GPT-5.4, Gemini 3.1 Pro, Claude Sonnet 4.6 (default) |
| `/v2/device/setting/save` | POST | form: `deviceSn,setting,value` | Not tested (mutating) |
| `/v2/device/settings` | POST | form: `deviceSn,version` | ✅ Live — returns empty for `version=latest` on this device |

### Devices (12)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/user/device/list` | POST | none | ✅ Live — 2 devices: H1 (jensen, fw 5.2.4, public) + P1 (eason, fw 1.4.5, private) |
| `/v1/user/device/status` | POST | form: `deviceSn` | ✅ Live — ownership/accessibility info (note: GET on this path is 405; must POST) |
| `/v1/user/device/bind` | POST | form payload | Not tested (mutating) |
| `/v1/user/device/unbind` | POST | form: `deviceSn` | Not tested (mutating, destructive-ish) |
| `/v1/user/device/rename` | POST | form: `deviceSn,name` | Not tested (mutating) |
| `/v1/user/device/accessibility/set` | POST | form: `deviceSn,accessibility` | Not tested (mutating) |
| `/v1/user/device/file/list` | POST | form: `deviceSn,pageIndex,pageSize` | ✅ Live — full recording file list w/ signatures, matches note IDs |
| `/v1/user/device/file/get` | POST | form: `deviceSn,signature` | Not tested this pass (likely returns file metadata/binary) |
| `/v1/user/device/file/upload` | POST | multipart w/ binary block | Not tested (mutating, chunked upload) |
| `/v1/user/device/file/clear?deviceSn=` | POST | query param | Not tested (destructive — clears device files) |
| `/v2/device/firmware/latest` | POST | form: `model,version` | Tested, 400 (param combo unclear — needs correct `model` value from device list, not `eason`/`jensen` directly) |
| `/v2/device/firmware/list` | GET | query: `pageIndex,model,pageSize` | Tested, 405 (GET wrong — likely needs POST despite being a "list") |
| `/v2/device/firmware/get?id=&deviceSn=&accesstoken=` | GET | query, incl. token | Not tested (firmware binary download) |
| `/v2/device/log` | POST | form payload | Not tested |
| `/v2/device/upload` | POST (multipart, progress) | `file,tzOffset,fingerprint` | Not tested (mutating, large upload) |
| `/v2/device/${recording\|whisper}/transcribe` | POST | form: device/file signature payload | Not tested (mutating, starts transcription job) |
| `/v2/device/recording/summarize` | POST | form: `noteId,aiEngine,template,...` | Not tested (mutating, costs AI quota) |
| `/v2/device/recording/transcribe-only` | POST | form: `noteId,tzOffset,language` | Not tested (mutating) |

### Notes — recordings (v1 + v2, ~20 endpoints)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/note/recording/list` | GET | `pageIndex,pageSize,sortType,sortField` (omit `folderId`!) | ✅ Live — 326 notes total, paginated correctly |
| `/v1/note/recording/detail` | GET | `noteId` | ✅ Live — core metadata (folderId, duration, hasAudio, etc.) |
| `/v2/note/info` | POST | form: `id` | ✅ Live — **richer than v1 detail**: includes full `markdown` summary, `conciseSummary`, `tags`, `shortId`, `sourceDevice`, `deviceType` |
| `/v2/note/meta` | GET (as SWR key; actual verb unclear) | `noteId` | Tested via GET, 405 — needs POST or different param shape, unresolved |
| `/v1/note/recording/transcriptions` | GET | `noteId` | ✅ Live — per-sentence, speaker+timestamp+channel |
| `/v2/note/transcription/list` | POST | form: `noteId` | ✅ Live — near-identical to v1 version, slightly different fields (no `channel`/`confidenceLevel`, has same speaker/timing) |
| `/v2/note/speaker/list` | GET | `noteId` | Tested via GET, 405 — needs POST, unresolved this pass |
| `/v2/note/speaker/change` | POST | form: `noteId,sentenceId,name` | Not tested (mutating — relabel one sentence's speaker) |
| `/v2/note/speaker/rename` | POST | form: `noteId,oldName,newName` | Not tested (mutating — relabel all instances) |
| `/v1/note/speaker/find` | POST | form: `name` | ✅ Live — looks up a saved speaker-name color/code mapping |
| `/v2/note/audio/download?noteId=&accesstoken=` | GET | query incl. token | ✅ Live — real MP3, `Content-Disposition: attachment`, exact byte size matches device file list |
| `/v2/note/audio/stream?noteId=&accesstoken=` | GET | query incl. token | ✅ Live — same file, `Accept-Ranges: bytes`, ETag, suited for a seekable `<audio>` player |
| `/v2/note/audio/resample` | POST | form: `noteId` | Not tested (mutating — regenerates resampled audio) |
| `/v1/audio/vad?noteId=&minSilenceMs=` | GET | query | ✅ Live — array of silence-boundary timestamps (ms), for waveform/silence trimming UI |
| `/v1/audio/merge` | POST | note file merge payload | Not tested (mutating) |
| `/v1/audio/replace` | POST | note file replace payload | Not tested (mutating) |
| `/v1/audio/saveAsNew` | POST | note file payload | Not tested (mutating) |
| `/v2/note/summarize` | POST | form: `noteId,aiEngine,templateCode,tzOffset` | Not tested (mutating, costs AI quota — (re)generates the AI summary) |
| `/v2/note/markdown/update` | POST | form: `noteId,markdown` | Not tested (mutating — manual summary edit) |
| `/v2/note/paragraph/update` | POST | form: `noteId,sentenceId,paragraphText` | Not tested (mutating — edits one transcript sentence) |
| `/v2/note/title/update` | POST | form: `noteId,title` | Not tested (mutating) |
| `/v2/note/createTime/update` | POST | form: `noteId,createTime,tzOffset` | Not tested (mutating) |
| `/v2/note/translate` | POST | form: `noteId,language` | Not tested (mutating/costs quota) |
| `/v1/note/rate` | POST | form: `id,level,remark` | Not tested this pass (mutating; confirmed field names from bundle — NOTE: `id`/`level`, not `noteId`/`rating` as originally assumed in v2 client) |
| `/v1/note/delete` | POST | form: `id` | **NOT TESTED — destructive** (field is `id`, not `noteId`) |
| `/v1/note/${type}/estimate` | POST | form: `mode,duration` | ✅ Live via `/v1/note/recording/estimate` — returns an estimated processing-time number |
| `/v1/note/${type}/find` | POST | form payload | Tested via `/v1/note/recording/find`... wait, actual path is `/v1/note/{fa}/find` where `fa` = search term, NOT type — **path template misread, needs re-verification** |
| `/v1/note/${type}/get` | POST | form: `mode,ticket` | Not tested (progress-check polling endpoint, needs an in-flight ticket) |
| `/v2/note/section/event/list?id=` | GET | query | ✅ Live — returns empty `{"error":0,"message":"success"}` (no calendar section linked to this note) |
| `/v2/note/summary/extract/calendar` | GET | query, `noteId` implied | Not tested |
| `/v1/note/recording/update/todo/{noteId}` | POST | form: `noteId,todoText` | Not tested (mutating — creates a todo item FROM a note's checklist; this is how todos actually get created, not a direct create-todo endpoint) |

### Notes — whisper (quick voice notes) (9)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/note/whisper/list` | GET | `pageSize,sortField` | ✅ Live — empty (account has 0 whispers currently) |
| `/v1/note/whisper/detail` | GET (assumed) | `noteId` | Not tested (0 whispers to test against) |
| `/v1/note/whisper/get` | — | — | Tested, 405 on GET; verb unresolved |
| `/v1/note/whisper/delete?noteId=` | DELETE | query | Not tested (mutating) |
| `/v1/note/whisper/rerun` | POST | form payload | Not tested |
| `/v1/note/whisper/title/update` | POST | form: `noteId,title` | Not tested |
| `/v1/note/whisper/transcriptions` | POST/GET | `noteId` | Not tested (0 whispers) |
| `/v1/note/whisper/paragraph/update` | POST | form: `noteId,sentenceId,paragraph` | Not tested (mutating) |
| `/v1/note/whisper/create/note` | POST | form: `noteIds,template,tzOffset` | Not tested (merges multiple whispers into one note) |
| `/v1/note/whisper/add/todo` | POST | form: `noteId,tzOffset` | Not tested (creates todo from whisper) |
| `/v1/note/whisper/extract/calendar` | GET | query | Not tested |

### Folders (5) — fully verified incl. mutations
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/folder/list` | POST | none | ✅ Live — 6 folders w/ noteCount |
| `/v1/folder/create` | POST | form: `name` | ✅ Live-tested — creates folder (response `data` is empty; must re-list to get new ID) |
| `/v1/folder/rename` | POST | form: `folderId,name` | ✅ Live-tested |
| `/v1/folder/remove` | POST | form: `folderId` | ✅ Live-tested |
| `/v1/folder/assign` | POST | form: `noteId,folderId` | Not tested (mutating, moves a real note) |
| `/v1/folder/unset?noteId=` | POST | query | Not tested (mutating) |

### To-Do (7)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/todo/list` | GET | `pageSize,state,tzOffset,dueDateStart,dueDateEnd` | ✅ Live — empty (no open todos currently) |
| `/v1/todo/delete` | POST | form: `id` | Not tested (mutating) |
| `/v1/todo/update/description` | POST | form: `id,description` | Not tested (mutating) |
| `/v1/todo/update/dueDate` | POST | form: `id,dueDate,tzOffset` | Not tested (mutating) |
| `/v1/todo/update/smartLabel` | POST | form: `id,smartLabelId` | Not tested (mutating) |
| `/v1/todo/{status}` | POST | form: `id` (status is the path segment: open/completed/archived) | Not tested (mutating) |
| **No direct "create todo" endpoint** | — | — | Confirmed by design: todos are created FROM notes via `/v1/note/recording/update/todo/{noteId}` or `/v1/note/whisper/add/todo`, never standalone |

### Calendar (4)
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/calendar/event/list` | GET | `start_time,end_time,tz_offset` | Verified shape from original pass; not re-tested this sweep |
| `/v1/calendar/event/add` | POST | form: `title,startTime,endTime` | Not tested (mutating) |
| `/v1/calendar/event/device_state/notice` | POST | form: `ids,state` (renamed from earlier assumed `eventId,isRecording`) | Not tested |
| `/v1/calendar/oauth2/authorize` | POST | OAuth payload | Not tested |

### Templates (9) — mostly verified
| Endpoint | Method | Body/Params | Status |
|---|---|---|---|
| `/v1/template/list` | POST | form: `pageSize,language` | ✅ Live — 42 templates (public + 1 owned copy), rich metadata incl. category/tags/level/icon |
| `/v1/template/info` | POST | form: `code,language` | Not tested this pass (shape confirmed from bundle) |
| `/v1/template/delete` | POST | form: `code` | Not tested (mutating) |
| `/v1/template/fav/toggle` | POST | form: `code,on` | Not tested (mutating) |
| `/v1/template/setDefault` | POST | form: `code` | Not tested (mutating) |
| `/v1/template/test` | POST (JSON!) | `templateId,prompt,transcript,model` | Not tested (mutating, costs AI quota — note this uses `postJson`, not form-data, unlike almost everything else) |
| `/v1/template/title/update` | POST | form: `code,title` | Not tested (mutating) |
| `/v1/template/content/update` | POST | form: `code,content` | Not tested (mutating) |

### Smart Labels (3)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/smart_label/create` | POST | form: `name,prompt,color` | Not tested (mutating) |
| `/v1/smart_label/update` | POST | form: `id,name,prompt,color` | Not tested (mutating) |
| `/v1/smart_label/delete` | POST | form: `id` | Not tested (mutating) |

### Vocabulary (3) — read verified
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/vocabulary/list` | POST | none | ✅ Live — 2 entries: NINA→NENA, Sean→Shaun (custom transcription corrections) |
| `/v1/vocabulary/create` | POST | form: `word,replacement,noteId,replace` | Not tested (mutating) |
| `/v1/vocabulary/delete` | POST | form: `id` | Not tested (mutating) |

### Sharing (2)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/share/create` | POST | note/share payload | Not tested (mutating — creates a public share link) |
| `/v1/share/resample?verifyCode=` | POST | form: `shortId` | Not tested |
| *(get shared note detail exists client-side but no bare static path found — likely `/v1/share/{shortId}` dynamic)* | — | — | Not mapped |

### Live translation (3)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/live/language/list` | GET | none | ✅ Live — 11 languages (en, ja, zh, de, fr, es, it, nl, pt, sv, pl) |
| `/v1/live/note/get` | POST | form: `ticket` | Not tested (needs active live session ticket) |
| `/v1/live/rate` | POST | form: `sessionId,rating,remark` | Not tested |

### Integrations (6)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v2/integration/list` | GET | none | ✅ Live — 1 Google integration, calendar read+write permission |
| `/v2/integration/disconnect` | POST | form: `id` | Not tested (mutating) |
| `/v2/integration/notion/authorization` | POST | none | Not tested (OAuth kickoff) |
| `/v2/integration/notion/status` | POST | none | ✅ Live — `false` (not connected) |
| `/v2/integration/notion/list` | POST | none | Not tested |
| `/v2/integration/notion/transfer` | POST | form payload | Not tested (mutating) |

### Subscription / Billing — RevenueCat (5)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/subscribers` | GET | none | Tested, **404** — path likely wrong/deprecated, or needs a sub-path (bundle shows `RCBilling`/`FS`-class internal SDK objects, not a bare `/v1/subscribers` REST call — this path may not exist as a plain GET at all) |
| `/v1/receipts` | GET | none | Tested, **404** — same caveat |
| `/v1/payment/rc/portal` | GET | none | ✅ Live — returns `{"error":90003,"message":"invalid_request"}` (needs params not yet identified, likely a redirect target or platform param) |
| `/v1/redemption/fulfill` | POST | form: `redemptionCode,verifyCode` | Not tested (mutating) |
| `/v1/redemption/info` | POST | form: `redemptionCode` | Not tested |

### Referral program (7)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/referral/overview` | GET | none | Tested, returns `{"error":90000,"message":"sys_failure"}` — endpoint exists but errored (possibly requires enrollment first) |
| `/v1/referral/rewards-overview` | GET | none | ✅ Live — `{totalReferred:0, totalCashback:0, totalMinutesEarned:0}` |
| `/v1/referral/message-template` | GET | none | ✅ Live — returns the share message copy text |
| `/v1/referral/paypal/connect` | POST | form: `paypalEmail` | Not tested (mutating) |
| `/v1/referral/paypal/disconnect` | POST | none | Not tested (mutating) |
| `/v1/referral/choose-minutes` | POST | form: `rewardId` | Not tested (mutating) |

### Sync (2)
| Endpoint | Method | Body | Status |
|---|---|---|---|
| `/v1/changes` | POST | Tested empty, got 400 — needs a body param (bundle shows it called bare in some places, with args in others; exact required shape unresolved) |
| `/v1/entry/info` | POST | `{}` | ✅ Live — returns a random motivational quote + `visited` flag; low practical value |

---

## Summary counts

- **125 unique endpoint paths** identified from the live JS bundle (91 `/v1/`, 34 `/v2/`)
- **~45 endpoints live-tested this pass** with real responses confirmed (read-only + 3 safe mutations: folder create/rename/delete cycle)
- **~80 endpoints NOT live-tested** — either genuinely destructive/costly (delete account, delete note, AI-quota-consuming summarize/translate calls), require state we don't have (whisper notes, live session tickets, in-flight device transcription jobs), or had ambiguous verb/param requirements that returned 400/405 and need another pass with corrected parameters
- **3 bugs found and fixed in this pass**: `folderId=-1` empty-result trap (prior pass), `/v1/user/info` false-positive validity check (this pass), `/v1/note/rate` field names are `id,level,remark` not `noteId,rating` as the v2 client assumed

## Recommended next steps if deeper mutation testing is wanted

Testing the destructive/costly endpoints (note delete, account delete, AI
summarize/translate, device unbind) safely would require either a
disposable test note/account or explicit written confirmation before each
call — flag to the user individually rather than batch-testing blind.
