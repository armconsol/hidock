"""
HiNotes API Client (Unofficial) - v2

A Python client for interacting with the HiNotes API (used with the HiDoc P1
USB audio transcription device). Reverse-engineered from the live app bundle
and confirmed against the real backend on 2026-09-02.

USE AT YOUR OWN RISK. This may violate HiNotes Terms of Service. Consider
contacting HiDock for official API access before using in production.

KEY CORRECTIONS FROM v1 (do not regress these):
1. Auth is NOT a cookie session and NOT a standard `Authorization: Bearer`
   header. The web app stores the raw token in `localStorage['accessToken']`
   and sends it back on every request as a custom header: `accesstoken: <token>`.
2. POST requests from the real client are sent as `multipart/form-data`
   (via FormData), not JSON. Sending JSON bodies gets a generic 400.
3. `/v1/user/signin` requires a Google reCAPTCHA v2 token
   (`{"email":..., "password":..., "captcha": <recaptcha_response>}`) and
   the server enforces it (confirmed: returns `{"error":10002,
   "message":"captcha_required"}` without one). There is NO way to fully
   automate this endpoint headlessly without a working captcha solve.
   >>> Practical workaround: log in manually in a real browser once, then
   >>> run `localStorage.getItem('accessToken')` in DevTools console and
   >>> pass that string into HiNotesClient(auth_token=...). Repeat when it
   >>> expires (401/403 on any call = dead token, unknown fixed TTL).
4. List/filter params matter: passing `folderId=-1` on
   `/v1/note/recording/list` returns EMPTY results, despite docs implying
   -1 means "all folders". Omit `folderId` entirely to get all notes.
5. Per-note detail is `GET /v1/note/recording/detail?noteId=<id>` (NOT
   `/v1/note/{id}/detail` as the raw bundle template suggests -- that
   template's first segment is a literal type keyword: "recording" or
   "whisper", not the note id).
6. Full per-sentence transcript (speaker-labelled, timestamped) is
   `GET /v1/note/recording/transcriptions?noteId=<id>`.
"""

import requests
from typing import Optional, Dict, List, Any
from datetime import datetime


class HiNotesClient:
    """
    Unofficial HiNotes API Client

    Usage:
        # Obtain a token once via manual browser login + DevTools:
        #   localStorage.getItem('accessToken')
        client = HiNotesClient(auth_token="<token from browser>")
        notes = client.list_notes()
        detail = client.get_note_detail(notes['content'][0]['id'])
        transcript = client.get_transcript(notes['content'][0]['id'])
    """

    BASE_URL = "https://hinotes.hidock.com/v1"

    def __init__(self, auth_token: Optional[str] = None):
        self.session = requests.Session()
        self.auth_token = auth_token

    def _headers(self, content_type: Optional[str] = None) -> Dict[str, str]:
        headers = {
            "Accept": "application/json",
            "Interface-Language": "en",
        }
        if self.auth_token:
            headers["accesstoken"] = self.auth_token
        if content_type:
            headers["Content-Type"] = content_type
        return headers

    def _get(self, endpoint: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        url = f"{self.BASE_URL}/{endpoint.lstrip('/')}"
        resp = self.session.get(url, params=params, headers=self._headers())
        resp.raise_for_status()
        return resp.json()

    def _post_form(self, endpoint: str, data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """POST as multipart/form-data, matching the real web client.
        `endpoint` starting with /v1/ or /v2/ is treated as absolute
        (relative to the host, not BASE_URL) so v2 endpoints work too."""
        if endpoint.startswith("/v1/") or endpoint.startswith("/v2/"):
            url = f"https://hinotes.hidock.com{endpoint}"
        else:
            url = f"{self.BASE_URL}/{endpoint.lstrip('/')}"
        clean = {k: v for k, v in (data or {}).items() if v is not None}
        resp = self.session.post(url, files={k: (None, str(v)) for k, v in clean.items()},
                                  headers=self._headers())
        resp.raise_for_status()
        return resp.json()

    def _post_json(self, endpoint: str, data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """POST with a genuine JSON body. A handful of endpoints require
        this instead of multipart/form-data (confirmed live: sending
        form-data to these gets a 415 Unsupported Media Type). Known
        JSON-body endpoints: /v1/note/whisper/paragraph/update,
        /v1/calendar/event/add, /v1/audio/merge|replace|saveAsNew,
        /v1/note/whisper/create/note. If a NEW endpoint 415s on
        _post_form, retry it here instead of assuming form-data always
        works."""
        if endpoint.startswith("/v1/") or endpoint.startswith("/v2/"):
            url = f"https://hinotes.hidock.com{endpoint}"
        else:
            url = f"{self.BASE_URL}/{endpoint.lstrip('/')}"
        clean = {k: v for k, v in (data or {}).items() if v is not None}
        resp = self.session.post(url, json=clean, headers=self._headers("application/json"))
        resp.raise_for_status()
        return resp.json()

    def is_token_valid(self) -> bool:
        """Cheap check: does the current token still work?

        IMPORTANT: do NOT use get_user_info()/`/v1/user/info` for this --
        confirmed live that with a dead token it silently returns
        error:0 for a freshly auto-provisioned anonymous Guest account
        instead of failing. `folder/list` correctly returns
        {"error":10000,"message":"session_timeout"} on a dead token, so
        we use that instead.
        """
        try:
            resp = self._post_form("/folder/list")
            return resp.get("error") == 0
        except requests.HTTPError:
            return False

    # --- Authentication -----------------------------------------------
    # NOTE: signin requires a live reCAPTCHA token; not practical to
    # automate headlessly. Get the token manually instead (see module
    # docstring). Left here for completeness / if you solve the captcha
    # some other way (e.g. a captcha-solving service you're authorized
    # to use).

    def authenticate_with_credentials(self, email: str, password: str, captcha: str) -> Dict[str, Any]:
        resp = self._post_form("/user/signin", {
            "email": email,
            "password": password,
            "captcha": captcha,
        })
        return resp

    def logout(self) -> Dict[str, Any]:
        return self._post_form("/user/logout")

    # --- User -----------------------------------------------------------

    def get_user_info(self) -> Dict[str, Any]:
        return self._post_form("/user/info")

    # --- Devices ----------------------------------------------------------

    def list_devices(self) -> Dict[str, Any]:
        return self._post_form("/user/device/list")

    def get_device_status(self) -> Dict[str, Any]:
        return self._get("/user/device/status")

    def list_device_files(self) -> Dict[str, Any]:
        return self._get("/user/device/file/list")

    # --- Folders ------------------------------------------------------

    def list_folders(self) -> Dict[str, Any]:
        return self._post_form("/folder/list")

    def create_folder(self, name: str) -> Dict[str, Any]:
        return self._post_form("/folder/create", {"name": name})

    # --- Notes (meeting recordings) ------------------------------------
    # IMPORTANT: do not pass folderId=-1 expecting "all" -- omit it.

    def list_notes(
        self,
        folder_id: Optional[str] = None,
        page_index: int = 0,
        page_size: int = 20,
        sort_type: str = "desc",
        sort_field: str = "createtime",
    ) -> Dict[str, Any]:
        params = {
            "pageIndex": page_index,
            "pageSize": page_size,
            "sortType": sort_type,
            "sortField": sort_field,
        }
        if folder_id is not None:
            params["folderId"] = folder_id
        return self._get("/note/recording/list", params=params)["data"]

    def list_all_notes(self, page_size: int = 50) -> List[Dict[str, Any]]:
        """Paginate through every note. Use this rather than a hardcoded
        page cap -- the note count grows over time."""
        all_notes: List[Dict[str, Any]] = []
        page = 0
        while True:
            result = self.list_notes(page_index=page, page_size=page_size)
            all_notes.extend(result.get("content", []))
            if result.get("last", True):
                break
            page += 1
        return all_notes

    def get_note_detail(self, note_id: str) -> Dict[str, Any]:
        """Metadata for one meeting recording (title, duration, folder, etc).
        For the full AI summary text, prefer get_note_info() (v2) which
        includes the rendered markdown summary; this v1 endpoint only has
        bare metadata fields."""
        return self._get("/note/recording/detail", params={"noteId": note_id})["data"]

    def get_note_info(self, note_id: str) -> Dict[str, Any]:
        """Richer note detail via v2 API: includes full `markdown` summary,
        `conciseSummary`, `tags`, `shortId`, `sourceDevice`, `deviceType`.
        Prefer this over get_note_detail() when you need the summary text."""
        return self._post_form("/v2/note/info", {"id": note_id})["data"]

    def get_transcript(self, note_id: str) -> List[Dict[str, Any]]:
        """Full per-sentence transcript with speaker labels and timestamps
        (beginTime/endTime in ms). Returns a list of segment dicts."""
        resp = self._get("/note/recording/transcriptions", params={"noteId": note_id})
        # Server wraps repeated <data> elements; requests/json client sees
        # this as {"data": [...]} once decoded from the XML-ish envelope --
        # if it comes back as a single dict, normalize to a list.
        data = resp.get("data", [])
        if isinstance(data, dict):
            data = [data]
        return data

    def download_audio(self, note_id: str, dest_path: str) -> str:
        """Download the note's audio file (MP3) to dest_path. Confirmed
        live: returns a real MP3 with Content-Disposition: attachment.
        Token is passed as a query param here since this is normally
        triggered via window.open() in the browser (no custom header)."""
        url = "https://hinotes.hidock.com/v2/note/audio/download"
        resp = self.session.get(url, params={"noteId": note_id, "accesstoken": self.auth_token}, stream=True)
        resp.raise_for_status()
        with open(dest_path, "wb") as f:
            for chunk in resp.iter_content(chunk_size=65536):
                f.write(chunk)
        return dest_path

    def get_audio_stream_url(self, note_id: str) -> str:
        """Build a seekable streaming URL (supports Range requests / ETag)
        for embedding in an <audio> player, e.g. for a web UI."""
        return (f"https://hinotes.hidock.com/v2/note/audio/stream"
                f"?noteId={note_id}&accesstoken={self.auth_token}")

    def delete_note(self, note_id: str) -> Dict[str, Any]:
        """NOT LIVE-TESTED (destructive). Confirmed field name from the
        JS bundle is `id`, NOT `noteId` -- the original client had this
        wrong."""
        return self._post_form("/note/delete", {"id": note_id})

    def rate_note(self, note_id: str, level: int, remark: Optional[str] = None) -> Dict[str, Any]:
        """NOT LIVE-TESTED. Confirmed field names from the JS bundle are
        `id`, `level`, `remark` -- NOT `noteId`/`rating` as the original
        client assumed."""
        return self._post_form("/note/rate", {"id": note_id, "level": level, "remark": remark})

    # --- Whisper notes (quick voice notes) -----------------------------

    def list_whispers(self, page_size: int = 20, sort_field: str = "create_time") -> Dict[str, Any]:
        return self._get("/note/whisper/list", params={
            "pageSize": page_size,
            "sortField": sort_field,
        })["data"]

    def get_whisper_detail(self, note_id: str) -> Dict[str, Any]:
        """GET /v1/note/whisper/detail?noteId= -- confirmed live (NOT POST,
        despite superficially resembling the recording-note pattern)."""
        return self._get("/note/whisper/detail", params={"noteId": note_id})["data"]

    def get_whisper_transcript(self, note_id: str) -> List[Dict[str, Any]]:
        """GET /v1/note/whisper/transcriptions?noteId= -- confirmed live."""
        resp = self._get("/note/whisper/transcriptions", params={"noteId": note_id})
        data = resp.get("data", [])
        return [data] if isinstance(data, dict) else data

    def update_whisper_title(self, note_id: str, title: str) -> Dict[str, Any]:
        """POST /v1/note/whisper/title/update?noteId=&title= -- params go
        in the query string, not the body. Confirmed live update+revert."""
        url = f"https://hinotes.hidock.com/v1/note/whisper/title/update"
        resp = self.session.post(url, params={"noteId": note_id, "title": title}, headers=self._headers())
        resp.raise_for_status()
        return resp.json()

    def update_whisper_paragraph(self, note_id: str, sentence_id: str, paragraph: str) -> Dict[str, Any]:
        """Requires a genuine JSON body -- confirmed live (form-data gets
        415 Unsupported Media Type on this one specific endpoint)."""
        return self._post_json("/v1/note/whisper/paragraph/update", {
            "noteId": note_id, "sentenceId": sentence_id, "paragraph": paragraph,
        })

    def add_whisper_to_todo(self, note_id: str, tz_offset: int = 0) -> Dict[str, Any]:
        """Creates a real todo item from a whisper note's content.
        Confirmed live -- this is the only way todos get created (no
        standalone create-todo endpoint exists)."""
        return self._post_form("/note/whisper/add/todo", {"noteId": note_id, "tzOffset": tz_offset})

    def extract_calendar_from_whisper(self, note_id: str) -> Dict[str, Any]:
        """GET /v1/note/whisper/extract/calendar?id= -- note the param is
        `id`, NOT `noteId`. Returns an AI-extracted calendar event
        suggestion (title/start/end/outline) from the whisper's content."""
        return self._get("/note/whisper/extract/calendar", params={"id": note_id})["data"]

    # --- To-Do ----------------------------------------------------------

    def list_todos(
        self,
        state: str = "open",
        page_size: int = 10,
        tz_offset: int = 0,
        due_date_start: Optional[datetime] = None,
        due_date_end: Optional[datetime] = None,
    ) -> Dict[str, Any]:
        params = {"pageSize": page_size, "state": state, "tzOffset": tz_offset}
        if due_date_start:
            params["dueDateStart"] = due_date_start.strftime("%Y-%m-%d %H:%M:%S")
        if due_date_end:
            params["dueDateEnd"] = due_date_end.strftime("%Y-%m-%d %H:%M:%S")
        return self._get("/todo/list", params=params)

    # --- Calendar ---------------------------------------------------------

    def list_calendar_events(self, start_time: datetime, end_time: datetime, tz_offset: int = 0) -> Dict[str, Any]:
        return self._get("/calendar/event/list", params={
            "start_time": start_time.strftime("%Y-%m-%d %H:%M:%S"),
            "end_time": end_time.strftime("%Y-%m-%d %H:%M:%S"),
            "tz_offset": tz_offset,
        })

    # --- Sync -------------------------------------------------------------

    def sync_changes(self) -> Dict[str, Any]:
        return self._post_form("/changes")

    def get_entry_info(self) -> Dict[str, Any]:
        return self._post_form("/entry/info")

    # --- Settings / templates / vocabulary (read-only, verified) --------

    def list_templates(self, page_size: int = 100, language: str = "en") -> Dict[str, Any]:
        return self._post_form("/template/list", {"pageSize": page_size, "language": language})["data"]

    def list_vocabulary(self) -> List[Dict[str, Any]]:
        resp = self._post_form("/vocabulary/list")
        data = resp.get("data", [])
        return [data] if isinstance(data, dict) else data

    def list_ai_engines(self) -> List[Dict[str, Any]]:
        resp = self._post_form("/user/setting/ai_engine/list")
        data = resp.get("data", [])
        return [data] if isinstance(data, dict) else data

    def get_user_setting(self, group: str, code: str) -> Any:
        return self._post_form("/user/setting/get", {"group": group, "code": code}).get("data")

    def list_devices_v1(self) -> List[Dict[str, Any]]:
        """Alias retained for clarity alongside list_devices(). Confirmed
        live: returns deviceSn, name, accessibility, firmwareVersion,
        deviceType per device."""
        resp = self._post_form("/user/device/list")
        data = resp.get("data", [])
        return [data] if isinstance(data, dict) else data

    def list_device_recordings(self, device_sn: str, page_index: int = 0, page_size: int = 10) -> Dict[str, Any]:
        """List raw recording files on a bound device (signature, fileName,
        matches noteId to the processed note). Different from
        list_device_files() which used a GET that isn't confirmed working;
        this POST form is live-verified."""
        return self._post_form("/user/device/file/list", {
            "deviceSn": device_sn, "pageIndex": page_index, "pageSize": page_size,
        })["data"]

    # --- Search, note metadata, speakers, sharing (verified round 2) ----

    def search_notes(self, keyword: str) -> Dict[str, Any]:
        """Full-text search across all recording notes (title, summary,
        transcription). Confirmed live: real path is
        POST /v1/note/recording/find (NOT a type-templated path). Matches
        include <b> highlight markup around the keyword in the response
        text -- strip if displaying to a user."""
        return self._post_form("/note/recording/find", {"keyword": keyword})["data"]

    def get_note_meta(self, note_id: str) -> Dict[str, Any]:
        """POST /v2/note/meta (confirmed POST, not GET -- GET 405s).
        Returns {aiModel, templateCode, templateTitle}."""
        return self._post_form("/v2/note/meta", {"noteId": note_id})["data"]

    def list_note_speakers(self, note_id: str) -> List[Dict[str, Any]]:
        """POST /v2/note/speaker/list (confirmed POST, not GET -- GET 405s)."""
        resp = self._post_form("/v2/note/speaker/list", {"noteId": note_id})
        data = resp.get("data", [])
        return [data] if isinstance(data, dict) else data

    def rename_speaker_in_sentence(self, note_id: str, sentence_id: str, name: str) -> Dict[str, Any]:
        """Relabels ONE sentence's speaker. Confirmed live (rename+revert
        tested clean). For relabeling ALL instances of a speaker name at
        once, use rename_speaker_globally() instead."""
        return self._post_form("/v2/note/speaker/change", {
            "noteId": note_id, "sentenceId": sentence_id, "name": name,
        })

    def rename_speaker_globally(self, note_id: str, old_name: str, new_name: str) -> Dict[str, Any]:
        """NOT LIVE-TESTED. Shape confirmed from bundle:
        POST /v2/note/speaker/rename {noteId,oldName,newName}."""
        return self._post_form("/v2/note/speaker/rename", {
            "noteId": note_id, "oldName": old_name, "newName": new_name,
        })

    def create_share_link(self, note_id: str, is_public: bool = True,
                           expire_time: str = "", verify_code: str = "") -> str:
        """Creates a public share link for a note. Confirmed live.
        IMPORTANT: calling this again on the same note generates a NEW
        shortId rather than updating/disabling the previous one -- there
        is no revoke/list-shares endpoint in this API. Returns the full
        share URL string."""
        resp = self._post_form("/v1/share/create", {
            "noteId": note_id, "isPublic": is_public,
            "expireTime": expire_time, "verifyCode": verify_code,
        })
        return resp.get("data", "")

    def get_shared_note(self, short_id: str, verify_code: str = "") -> Dict[str, Any]:
        """Confirmed live. Note: still requires the accesstoken header
        even though this is meant to be a public share page -- untested
        whether a genuinely logged-out browser session works differently."""
        params = {"shortId": short_id}
        if verify_code:
            params["verifyCode"] = verify_code
        return self._get("/share/note", params=params)["data"]

    def get_shared_transcript(self, short_id: str) -> List[Dict[str, Any]]:
        resp = self._get("/share/transcription/list", params={"shortId": short_id})
        data = resp.get("data", [])
        return [data] if isinstance(data, dict) else data

    # --- Smart labels (verified round 2) ---------------------------------

    def create_smart_label(self, name: str, prompt: str, color: str) -> str:
        """Confirmed live (create->update->delete cycle tested clean).
        Unlike folder/create, this returns the new ID directly in `data`."""
        resp = self._post_form("/smart_label/create", {"name": name, "prompt": prompt, "color": color})
        return resp.get("data", "")

    def update_smart_label(self, label_id: str, name: str, prompt: str, color: str) -> Dict[str, Any]:
        return self._post_form("/smart_label/update", {
            "id": label_id, "name": name, "prompt": prompt, "color": color,
        })

    def delete_smart_label(self, label_id: str) -> Dict[str, Any]:
        return self._post_form("/smart_label/delete", {"id": label_id})

    # --- Todo lifecycle (verified round 2, beyond list_todos) ------------

    def update_todo_description(self, todo_id: str, description: str) -> Dict[str, Any]:
        return self._post_form("/todo/update/description", {"id": todo_id, "description": description})

    def update_todo_due_date(self, todo_id: str, due_date: str, tz_offset: int = 0) -> Dict[str, Any]:
        """due_date format: 'YYYY-MM-DD HH:MM:SS'."""
        return self._post_form("/todo/update/dueDate", {"id": todo_id, "dueDate": due_date, "tzOffset": tz_offset})

    def change_todo_status(self, todo_id: str, status: str) -> Dict[str, Any]:
        """status is a literal URL path segment (confirmed: 'completed'
        works; 'open'/'archived' assumed from bundle, not independently
        tested)."""
        return self._post_form(f"/todo/{status}", {"id": todo_id})

    def delete_todo(self, todo_id: str) -> Dict[str, Any]:
        return self._post_form("/todo/delete", {"id": todo_id})


if __name__ == "__main__":
    import os
    token = os.environ.get("HINOTES_TOKEN")
    if not token:
        print("Set HINOTES_TOKEN env var (grab via browser DevTools: "
              "localStorage.getItem('accessToken')) to run this demo.")
    else:
        client = HiNotesClient(auth_token=token)
        print("Token valid:", client.is_token_valid())
        user = client.get_user_info()
        print("User:", user["data"]["name"], "-", user["data"]["totalNoteCount"], "notes")
        notes = client.list_all_notes()
        print(f"Fetched {len(notes)} notes total.")
        if notes:
            first = notes[0]
            print("Most recent:", first["title"])
            transcript = client.get_transcript(first["id"])
            print(f"Transcript has {len(transcript)} segments.")
