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
        """POST as multipart/form-data, matching the real web client."""
        url = f"{self.BASE_URL}/{endpoint.lstrip('/')}"
        clean = {k: v for k, v in (data or {}).items() if v is not None}
        resp = self.session.post(url, files={k: (None, str(v)) for k, v in clean.items()},
                                  headers=self._headers())
        resp.raise_for_status()
        return resp.json()

    def is_token_valid(self) -> bool:
        """Cheap check: does the current token still work?"""
        try:
            info = self.get_user_info()
            return info.get("error") == 0
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
        Does NOT include the summary text or transcript -- see
        list_notes()'s `conciseSummary` field for the AI summary, and
        get_transcript() for the full transcript."""
        return self._get("/note/recording/detail", params={"noteId": note_id})["data"]

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

    def delete_note(self, note_id: str) -> Dict[str, Any]:
        return self._post_form("/note/delete", {"noteId": note_id})

    def rate_note(self, note_id: str, rating: int) -> Dict[str, Any]:
        return self._post_form("/note/rate", {"noteId": note_id, "rating": rating})

    # --- Whisper notes (quick voice notes) -----------------------------

    def list_whispers(self, page_size: int = 20, sort_field: str = "create_time") -> Dict[str, Any]:
        return self._get("/note/whisper/list", params={
            "pageSize": page_size,
            "sortField": sort_field,
        })["data"]

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
