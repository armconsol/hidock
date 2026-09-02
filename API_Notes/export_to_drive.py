"""
HiNotes -> Google Drive export script.

Mirrors the HiNotes-to-Obsidian export (export_to_obsidian.py) but targets
a Google Drive folder tree instead of the local vault, and ALSO uploads
the raw audio recording for each note (Obsidian export does not).

Target root: https://drive.google.com/drive/folders/1gD9JR4eTqd8e75eKAbE70S1F_gamRFyd
("HiNote_Transcripts" folder in the user's Drive)

Structure created under that root:
    Summeries/<HiDock folder>/<title>.md   (uploaded as text/markdown)
    Transcripts/<HiDock folder>/<title>.md
    Recordings/<HiDock folder>/<title>.mp3  (real audio, streamed download+upload)

IDEMPOTENT: tracks exported note IDs (per artifact type) in
drive_export_manifest.json (same directory as this script) so reruns only
process NEW notes. Folder IDs are cached in the same manifest so repeat
runs don't recreate folders.

Requires:
- HINOTES_TOKEN env var (HiNotes accesstoken)
- Google Workspace OAuth already set up (~/.hermes/google_token.json)
- The gws_bridge-backed google_api.py CLI, invoked via subprocess with the
  python3.11 venv that has the google-api-python-client deps installed
  (this repo's system python3.9 cannot install those deps -- see
  ~/.hermes/gws_venv)

Usage:
    HINOTES_TOKEN=<token> python3 export_to_drive.py [--dry-run] [--limit N] [--skip-audio]
"""
import sys
import os
import json
import re
import time
import argparse
import subprocess
import tempfile

sys.path.insert(0, "/home/sarman/repos/hinotes/API_Notes")
from hinotes_client import HiNotesClient

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MANIFEST_PATH = os.path.join(SCRIPT_DIR, "drive_export_manifest.json")

ROOT_FOLDER_ID = "1gD9JR4eTqd8e75eKAbE70S1F_gamRFyd"  # HiNote_Transcripts

GWS_PYTHON = os.path.expanduser("~/.hermes/gws_venv/bin/python")
GAPI_SCRIPT = os.path.expanduser("~/.hermes/skills/productivity/google-workspace/scripts/google_api.py")


def sanitize_filename(name: str) -> str:
    name = re.sub(r'[\\/:*?"<>|]', "-", name)
    name = re.sub(r"\s+", " ", name).strip()
    return name[:180] or "Untitled"


def ms_to_hms(ms) -> str:
    if ms is None:
        return "00:00:00"
    s = int(ms) // 1000
    h, s = divmod(s, 3600)
    m, s = divmod(s, 60)
    return f"{h:02d}:{m:02d}:{s:02d}"


def load_manifest() -> dict:
    if os.path.exists(MANIFEST_PATH):
        with open(MANIFEST_PATH) as f:
            return json.load(f)
    return {
        "folder_ids": {},          # "Summeries/MSI" -> drive folder id
        "exported_summary_ids": [],
        "exported_transcript_ids": [],
        "exported_audio_ids": [],
    }


def save_manifest(manifest: dict) -> None:
    with open(MANIFEST_PATH, "w") as f:
        json.dump(manifest, f, indent=2)


def gapi(*args) -> dict:
    """Run the google_api.py CLI via the python3.11 venv with deps and
    return parsed JSON. Raises on non-zero exit."""
    cmd = [GWS_PYTHON, GAPI_SCRIPT] + list(args)
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        raise RuntimeError(f"gapi {args} failed: {result.stderr or result.stdout}")
    return json.loads(result.stdout)


def ensure_folder(manifest: dict, path_key: str, name: str, parent_id: str) -> str:
    """path_key is a manifest cache key like 'Summeries/MSI'. Creates the
    Drive folder if not already cached, returns its id."""
    if path_key in manifest["folder_ids"]:
        return manifest["folder_ids"][path_key]
    resp = gapi("drive", "create-folder", name, "--parent", parent_id)
    folder_id = resp["id"]
    manifest["folder_ids"][path_key] = folder_id
    save_manifest(manifest)
    return folder_id


def upload_text(content: str, filename: str, parent_id: str) -> dict:
    """Write content to a temp file and upload it as a native Google Doc
    (--convert-to-doc), so markdown syntax (headers/bold/lists) renders as
    real rich-text formatting and opening it in Docs edits in place instead
    of spawning a duplicate converted copy."""
    doc_name = filename[:-3] if filename.endswith(".md") else filename
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as tmp:
        tmp.write(content)
        tmp_path = tmp.name
    try:
        resp = gapi("drive", "upload", tmp_path, "--name", doc_name, "--parent", parent_id,
                    "--convert-to-doc", "--mime-type", "text/markdown")
        return resp
    finally:
        os.unlink(tmp_path)


def upload_audio(local_path: str, filename: str, parent_id: str) -> dict:
    return gapi("drive", "upload", local_path, "--name", filename, "--parent", parent_id)


def build_summary_md(note_id: str, info: dict, folder_name: str) -> str:
    title = info.get("title") or f"Untitled {note_id}"
    create_time_ms = info.get("createTime")
    duration_ms = info.get("duration")
    tags_csv = info.get("tags") or ""
    tag_list = [t.strip() for t in tags_csv.split(",") if t.strip()]
    member_count = info.get("memberCount")
    source_device = info.get("sourceDevice") or "unknown"
    markdown_body = info.get("markdown") or info.get("conciseSummary") or "*(no summary available)*"

    date_str = ""
    if create_time_ms:
        date_str = time.strftime("%Y-%m-%d", time.gmtime(int(create_time_ms) / 1000))

    header_lines = [
        f"# {title}",
        "",
        f"- **HiNotes ID**: {note_id}",
        f"- **Folder**: {folder_name}",
        f"- **Date**: {date_str}",
        f"- **Duration**: {duration_ms/1000:.0f}s" if duration_ms else "- **Duration**: unknown",
        f"- **Participants**: {member_count if member_count is not None else 'unknown'}",
        f"- **Source device**: {source_device}",
        "",
    ]
    if tag_list:
        header_lines.append("**Meeting tags:** " + ", ".join(tag_list))
        header_lines.append("")
    return "\n".join(header_lines) + markdown_body + "\n"


def build_transcript_md(note_id: str, title: str, folder_name: str,
                         create_time_ms, segments: list) -> str:
    date_str = ""
    if create_time_ms:
        date_str = time.strftime("%Y-%m-%d", time.gmtime(int(create_time_ms) / 1000))

    header_lines = [
        f"# {title} — Transcript",
        "",
        f"- **HiNotes ID**: {note_id}",
        f"- **Folder**: {folder_name}",
        f"- **Date**: {date_str}",
        "",
    ]
    body_lines = []
    for seg in segments:
        speaker = seg.get("speaker") or "Unknown"
        begin = ms_to_hms(seg.get("beginTime"))
        sentence = (seg.get("sentence") or "").strip()
        if sentence:
            body_lines.append(f"**[{begin}] {speaker}:** {sentence}")
            body_lines.append("")
    return "\n".join(header_lines + body_lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--limit", type=int, default=None)
    parser.add_argument("--skip-audio", action="store_true",
                         help="Skip audio download/upload (much faster; do summaries+transcripts only)")
    args = parser.parse_args()

    token = os.environ.get("HINOTES_TOKEN")
    if not token:
        print("ERROR: set HINOTES_TOKEN env var")
        sys.exit(1)

    client = HiNotesClient(auth_token=token)
    if not client.is_token_valid():
        print("ERROR: HINOTES_TOKEN is invalid/expired.")
        sys.exit(1)

    manifest = load_manifest()
    exported_summary = set(manifest["exported_summary_ids"])
    exported_transcript = set(manifest["exported_transcript_ids"])
    exported_audio = set(manifest["exported_audio_ids"])

    print(f"Already exported -- summaries: {len(exported_summary)}, "
          f"transcripts: {len(exported_transcript)}, audio: {len(exported_audio)}")

    # Ensure top-level Summeries/Transcripts/Recordings folders exist
    if not args.dry_run:
        summ_root_id = ensure_folder(manifest, "Summeries", "Summeries", ROOT_FOLDER_ID)
        trans_root_id = ensure_folder(manifest, "Transcripts", "Transcripts", ROOT_FOLDER_ID)
        rec_root_id = ensure_folder(manifest, "Recordings", "Recordings", ROOT_FOLDER_ID)
    else:
        summ_root_id = trans_root_id = rec_root_id = "DRY_RUN"

    # Folder membership map (discovered live, never hardcoded)
    folders_resp = client._post_form("/folder/list").get("data", [])
    if isinstance(folders_resp, dict):
        folders_resp = [folders_resp]
    folder_map = {f["id"]: f["name"] for f in folders_resp}

    note_to_folder = {}
    for fid, fname in folder_map.items():
        page = 0
        while True:
            result = client.list_notes(folder_id=fid, page_index=page, page_size=50)
            for n in result.get("content", []):
                note_to_folder[n["id"]] = fname
            if result.get("last", True):
                break
            page += 1

    all_notes = client.list_all_notes(page_size=50)
    print(f"Total notes in account: {len(all_notes)}")

    new_notes = [n for n in all_notes
                 if n["id"] not in exported_summary
                 or n["id"] not in exported_transcript
                 or (not args.skip_audio and n["id"] not in exported_audio)]
    print(f"Notes needing at least one new artifact: {len(new_notes)}")

    if args.limit:
        new_notes = new_notes[: args.limit]
        print(f"Limiting this run to {len(new_notes)} notes")

    errors = []
    subfolder_cache = {}  # e.g. "Summeries/MSI" -> folder_id, cached in manifest too

    def get_subfolder(kind_root_id, kind_key, folder_name):
        cache_key = f"{kind_key}/{folder_name}"
        if cache_key in subfolder_cache:
            return subfolder_cache[cache_key]
        fid = ensure_folder(manifest, cache_key, folder_name, kind_root_id)
        subfolder_cache[cache_key] = fid
        return fid

    for i, note in enumerate(new_notes):
        note_id = note["id"]
        title = note.get("title") or f"Untitled {note_id}"
        folder_name = note_to_folder.get(note_id, "Uncategorized")
        print(f"[{i+1}/{len(new_notes)}] {folder_name}/{title}")
        safe_title = sanitize_filename(title)

        need_summary = note_id not in exported_summary
        need_transcript = note_id not in exported_transcript
        need_audio = (not args.skip_audio) and note_id not in exported_audio

        if not (need_summary or need_transcript or need_audio):
            continue

        info = None
        if need_summary:
            try:
                info = client.get_note_info(note_id)
            except Exception as e:
                print(f"  ERROR fetching note info: {e}")
                errors.append({"note_id": note_id, "title": title, "stage": "info", "error": str(e)})
                need_summary = False

        if need_summary and info is not None:
            md = build_summary_md(note_id, info, folder_name)
            if args.dry_run:
                print(f"  [DRY RUN] would upload summary: Summeries/{folder_name}/{safe_title}.md")
            else:
                try:
                    fid = get_subfolder(summ_root_id, "Summeries", folder_name)
                    upload_text(md, f"{safe_title}.md", fid)
                    exported_summary.add(note_id)
                    manifest["exported_summary_ids"] = list(exported_summary)
                    save_manifest(manifest)
                except Exception as e:
                    print(f"  ERROR uploading summary: {e}")
                    errors.append({"note_id": note_id, "title": title, "stage": "upload_summary", "error": str(e)})

        if need_transcript:
            try:
                transcript = client.get_transcript(note_id)
            except Exception as e:
                print(f"  WARNING transcript fetch failed: {e}")
                transcript = []
            md = build_transcript_md(note_id, title, folder_name, note.get("createTime"), transcript)
            if args.dry_run:
                print(f"  [DRY RUN] would upload transcript: Transcripts/{folder_name}/{safe_title}.md")
            else:
                try:
                    fid = get_subfolder(trans_root_id, "Transcripts", folder_name)
                    upload_text(md, f"{safe_title}.md", fid)
                    exported_transcript.add(note_id)
                    manifest["exported_transcript_ids"] = list(exported_transcript)
                    save_manifest(manifest)
                except Exception as e:
                    print(f"  ERROR uploading transcript: {e}")
                    errors.append({"note_id": note_id, "title": title, "stage": "upload_transcript", "error": str(e)})

        if need_audio:
            if args.dry_run:
                print(f"  [DRY RUN] would download+upload audio: Recordings/{folder_name}/{safe_title}.mp3")
            else:
                tmp_audio = None
                try:
                    tmp_audio = tempfile.mktemp(suffix=".mp3")
                    client.download_audio(note_id, tmp_audio)
                    fid = get_subfolder(rec_root_id, "Recordings", folder_name)
                    upload_audio(tmp_audio, f"{safe_title}.mp3", fid)
                    exported_audio.add(note_id)
                    manifest["exported_audio_ids"] = list(exported_audio)
                    save_manifest(manifest)
                except Exception as e:
                    print(f"  ERROR audio (download or upload): {e}")
                    errors.append({"note_id": note_id, "title": title, "stage": "audio", "error": str(e)})
                finally:
                    if tmp_audio and os.path.exists(tmp_audio):
                        os.unlink(tmp_audio)

    print(f"\nDone. Summaries now: {len(exported_summary)}, "
          f"Transcripts: {len(exported_transcript)}, Audio: {len(exported_audio)}")
    if errors:
        print(f"{len(errors)} errors:")
        for e in errors:
            print(f"  [{e['stage']}] {e['title']}: {e['error']}")
        with open(os.path.join(SCRIPT_DIR, "drive_export_errors.json"), "w") as f:
            json.dump(errors, f, indent=2)


if __name__ == "__main__":
    main()
