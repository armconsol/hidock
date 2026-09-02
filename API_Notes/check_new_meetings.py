"""
Report HiNotes meetings that are NOT yet assigned to any folder ("Uncategorized").

For each one, prints: note id, title, duration, participant count, current
AI template, and a content preview (from the AI summary markdown) -- enough
for a human or an LLM-driven agent to decide:
  (a) which EXISTING folder it belongs in (never invent a folder name that
      doesn't already match the account's conventions unless truly nothing
      fits), and
  (b) whether the default/generic AI summary template should be regenerated
      with a better-fitting one (e.g. "General Meeting" -> "Client Meeting",
      "Stand Up Meeting", "Project Sync", etc.)

This script does NOT classify or mutate anything itself -- it only reports.
Classification (folder assignment + template regeneration) is a judgment
call made by whoever/whatever consumes this output, via:
  - POST /v1/folder/create  (form: name)              -- new folder if needed
  - POST /v1/folder/assign  (form: noteId, folderId)   -- assign to folder
  - POST /v2/note/summarize (form: noteId, aiEngine, templateCode, tzOffset)
                                                        -- regenerate summary

Usage:
    HINOTES_TOKEN=<token> python3 check_new_meetings.py
"""
import sys
import os
import json

sys.path.insert(0, "/home/sarman/repos/hinotes/API_Notes")
from hinotes_client import HiNotesClient


def main():
    token = os.environ.get("HINOTES_TOKEN")
    if not token:
        print("ERROR: set HINOTES_TOKEN env var")
        sys.exit(1)

    client = HiNotesClient(auth_token=token)
    if not client.is_token_valid():
        print("TOKEN_INVALID: HiNotes accesstoken is expired/invalid. "
              "Ask Shaun to log into https://hinotes.hidock.com in a browser, "
              "grab a fresh token via DevTools (localStorage.getItem('accessToken')), "
              "and update HINOTES_TOKEN in /home/sarman/repos/hinotes/.env")
        sys.exit(2)

    # Discover current folders (never hardcode -- account may have new ones)
    folders_resp = client._post_form("/folder/list").get("data", [])
    if isinstance(folders_resp, dict):
        folders_resp = [folders_resp]
    folders = [{"id": f["id"], "name": f["name"]} for f in folders_resp]
    folder_ids = {f["id"] for f in folders}

    note_to_folder = {}
    for f in folders:
        page = 0
        while True:
            result = client.list_notes(folder_id=f["id"], page_index=page, page_size=50)
            for n in result.get("content", []):
                note_to_folder[n["id"]] = f["id"]
            if result.get("last", True):
                break
            page += 1

    all_notes = client.list_all_notes(page_size=50)
    uncategorized = [n for n in all_notes if n["id"] not in note_to_folder]

    # Available summary templates (for regeneration decisions)
    tmpl_resp = client._post_form("/template/list", {"pageSize": 100, "language": "en"})
    templates = tmpl_resp.get("data", {}).get("content", [])
    template_list = [{"code": t.get("code"), "title": t.get("title"), "category": t.get("category")}
                      for t in templates]

    report = {
        "existing_folders": folders,
        "available_templates": template_list,
        "uncategorized_count": len(uncategorized),
        "uncategorized_notes": [],
    }

    for n in uncategorized:
        note_id = n["id"]
        try:
            info = client.get_note_info(note_id)
        except Exception as e:
            report["uncategorized_notes"].append({
                "id": note_id, "title": n.get("title"), "error": str(e),
            })
            continue

        meta = client._post_form("/v2/note/meta", {"noteId": note_id}).get("data", {})
        markdown = info.get("markdown") or info.get("conciseSummary") or ""
        report["uncategorized_notes"].append({
            "id": note_id,
            "title": info.get("title"),
            "duration_s": (info.get("duration") or 0) / 1000,
            "member_count": info.get("memberCount"),
            "current_template": meta.get("templateTitle"),
            "current_template_code": meta.get("templateCode"),
            "summary_preview": markdown[:600],
        })

    print(json.dumps(report, indent=2))
    if report["uncategorized_count"] == 0:
        print("\nNo uncategorized notes -- nothing to classify this run.", file=sys.stderr)


if __name__ == "__main__":
    main()
