# HiNotes API — Round 2 Verification Addendum (2026-09-02, using real Whisper note)

> Supplements `HiNotes_API_Verified_Map.md`. This pass used the user's
> freshly-recorded Whisper note (`6179309984027910144`, "Audio Equipment
> Test") plus safe create→verify→revert/delete cycles on smart labels,
> vocabulary, todos, speaker names, and share links. All test artifacts
> were cleaned up / reverted after verification.

## Newly confirmed endpoints

| Endpoint | Method | Body/Params | Result |
|---|---|---|---|
| `/v1/note/whisper/detail` | **GET** | `noteId` | ✅ Full whisper metadata (corrects earlier "POST assumed" note) |
| `/v1/note/whisper/transcriptions` | **GET** | `noteId` | ✅ Per-sentence transcript for whisper notes |
| `/v1/note/whisper/get` | POST | form: `mode,ticket` (whisper progress-poll) | ✅ Shape confirmed (`no_such_note` on a dead/nonexistent ticket = correct behavior, not a param error) |
| `/v1/note/whisper/extract/calendar` | GET | query: `id` (not `noteId`!) | ✅ Returns an AI-extracted calendar-event suggestion from the whisper content |
| `/v1/note/whisper/title/update` | POST, **query string** (not form body) | `?noteId=&title=` | ✅ Live-tested update + revert |
| `/v1/note/whisper/paragraph/update` | POST, **genuine JSON body** (`Content-Type: application/json`) | `{noteId,sentenceId,paragraph}` | ✅ Live-tested update + revert — this is a real exception to the form-data rule |
| `/v1/note/whisper/add/todo` | POST | form: `noteId,tzOffset` | ✅ Created a real todo from the whisper note |
| `/v1/note/whisper/delete` | **HTTP DELETE** (not POST!) | query: `noteId` | Confirmed shape from bundle (`deleter()` helper uses `method:"DELETE"`); not fired (would remove the user's real note) |
| `/v1/note/whisper/rerun` | GET | query: `noteId` | Confirmed shape from bundle (costs AI quota — not fired) |
| `/v2/note/meta` | **POST** (not GET — corrects earlier 405) | form: `noteId` | ✅ Returns `aiModel, templateCode, templateTitle` for the note |
| `/v2/note/speaker/list` | **POST** (not GET — corrects earlier 405) | form: `noteId` | ✅ Returns all named speakers for the note |
| `/v2/note/speaker/change` | POST | form: `noteId,sentenceId,name` | ✅ Live-tested rename + revert on a real transcript segment |
| `/v1/note/recording/find` | POST | form: `keyword` (full-text search, NOT `/v1/note/{type}/find` as earlier assumed — path is literally `/v1/note/recording/find`) | ✅ Real full-text search across all notes, returns matches with `<b>` highlight markup in title/summary/transcription |
| `/v1/changes` | POST | form: `pageIndex` | ✅ **Corrected earlier assumption** — this is the app changelog/version-check feed (returns version history + release notes), NOT a data-sync endpoint |
| `/v1/referral/overview` | GET | none | ✅ Retried clean this pass — returns referral link/code/enrolled status (earlier `sys_failure` was likely a transient blip) |
| `/v1/payment/rc/portal` | GET | none | Confirmed `error:90003 invalid_request` is expected/correct — this account is on a lifetime plan (not RevenueCat-billed), and the bundle's own error-handling for code `16014` maps to "manage portal not available" for exactly this case |
| `/v1/share/create` | POST | form: `noteId,isPublic,expireTime,verifyCode` | ✅ Created 2 real share links against a recording note; **note**: calling again with `isPublic=false` generates a NEW shortId rather than disabling the old one — sharing appears to have no "list my shares" or "revoke" endpoint in this bundle (create-only/regenerate model) |
| `/v1/share/note` | GET | query: `shortId,verifyCode` | ✅ Returns full note + speaker list + field-label map for the shared note (still requires `accesstoken` header even for a "public" share — unexpected, worth re-checking without any session at all from a clean browser) |
| `/v1/share/transcription/list` | GET | query: `shortId` | ✅ Returns full transcript for the shared note |
| `/v1/smart_label/create` | POST | form: `name,prompt,color` | ✅ Full CRUD cycle (create→update→delete) verified clean; **note**: unlike folder/create, this returns the new ID directly in `data` |
| `/v1/smart_label/update` | POST | form: `id,name,prompt,color` | ✅ |
| `/v1/smart_label/delete` | POST | form: `id` | ✅ |
| `/v1/vocabulary/create` | POST | form: `word,replacement,replace` (noteId optional) | ✅ Full create→delete cycle verified clean |
| `/v1/vocabulary/delete` | POST | form: `id` | ✅ |
| `/v1/todo/update/description` | POST | form: `id,description` | ✅ |
| `/v1/todo/update/dueDate` | POST | form: `id,dueDate,tzOffset` | ✅ |
| `/v1/todo/{status}` | POST | form: `id` (status literal in path, e.g. `/v1/todo/completed`) | ✅ Confirmed `completed` works |
| `/v1/todo/delete` | POST | form: `id` | ✅ Full todo lifecycle (create-from-note → update description → update due date → mark completed → delete) verified clean |
| `/v1/template/info` | POST | form: `code,language` | ✅ Returns full template with `prompt` field (the actual AI prompt text) |
| `/v1/folder/unset` | POST | query: `noteId` | ✅ Returns `permission_denied` for a whisper note (expected — whispers aren't folder-assignable); untested against a real folder-assigned recording note |

## Corrected assumptions from Round 1

1. **`/v1/changes` is the app changelog feed**, not a generic data-sync
   endpoint as originally guessed from its name. Returns version history
   with `changeLogs` markdown per release. Low practical value for a
   client — skip it in the wrapper.
2. **Request-body format is NOT a clean "form-data vs JSON" split by
   convention** — it's determined by which internal JS helper
   (`poster` vs `post`, confusingly near-opposite of intuition) each
   endpoint happens to use, and there are real per-endpoint exceptions:
   - `poster()` = genuine JSON body (used by e.g. `whisper/paragraph/update`,
     `calendar/event/add`, `audio/merge`)
   - `post()` = FormData (used by almost everything else, including
     `user/signin`, `note/rate`, `folder/create`)
   - A same-named function can ALSO be called with query-string params
     baked into the URL and an empty/no body (e.g. `whisper/title/update`,
     `folder/unset`) — these look like "POST with no body" rather than
     form-data OR JSON.
   **Practical implication**: don't assume a blanket rule for new/untested
   endpoints — check the specific call site in the bundle, or try
   form-data first (works for the majority) and fall back to JSON body
   if you get a 415 Unsupported Media Type (that status code is the
   reliable tell — confirmed live on `whisper/paragraph/update`).
3. **`/v2/note/meta` and `/v2/note/speaker/list` are POST, not GET** —
   the SWR hook key array in the bundle made them look like simple GETs;
   the actual fetcher function POSTs form-data. GET returns a clean 405
   on these (a good diagnostic signal generally: 405 means "right idea,
   wrong verb," 400 usually means "right verb, wrong/missing params").
4. **Sharing has no revoke/list-shares endpoint** in this app version —
   `share/create` is create-or-regenerate only. If the user wants to
   audit/kill old share links, that would need to go through HiDock
   support or isn't exposed client-side at all.

## Still not independently tested (destructive, quota-costing, or requiring unavailable state)

- `note/whisper/delete`, `note/delete`, `user/delete` — destructive
- `note/whisper/rerun`, `v2/note/summarize`, `v2/note/translate`,
  `v2/device/recording/summarize`, `v2/device/recording/transcribe-only`
  — consume AI/transcription quota
- `user/device/bind/unbind/rename`, `device/file/upload`,
  `device/accessibility/set` — require/affect real hardware state
- `live/note/get`, `live/rate` — require an active live-translation
  session ticket
- `user/password/update`, `user/register`, `user/reset/*`,
  `user/email/verification/*` — security-sensitive account operations

These remain flagged individually in `HiNotes_API_Verified_Map.md` as
"not tested" with their confirmed shape from the bundle; testing them
would need explicit per-call confirmation from the user given their
destructive/costly/account-security nature.
