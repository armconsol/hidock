"""
HiNotes -> Obsidian export script.

Exports every HiNotes meeting note's AI summary (markdown) and full
transcript into the Obsidian vault, mirroring HiDock's own folder
structure:
    Meetings/Summeries/<HiDock folder>/<title>.md
    Meetings/Transcripts/<HiDock folder>/<title>.md

IDEMPOTENT: tracks exported note IDs in export_manifest.json (in this
script's directory) so reruns only fetch/write NEW notes. Safe to
re-run on a cron/schedule as new HiNotes recordings appear -- never
hardcode a page/count ceiling (note count grows over time).

Usage:
    HINOTES_TOKEN=<token> python3 export_to_obsidian.py [--dry-run] [--limit N]
"""
import sys
import os
import json
import re
import glob
import argparse
import time

sys.path.insert(0, "/home/sarman/repos/hinotes/API_Notes")
from hinotes_client import HiNotesClient

VAULT = "/home/sarman/repos/obsidian"
SUMM_ROOT = os.path.join(VAULT, "Meetings", "Summeries")
TRANS_ROOT = os.path.join(VAULT, "Meetings", "Transcripts")
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MANIFEST_PATH = os.path.join(SCRIPT_DIR, "export_manifest.json")


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
            m = json.load(f)
    else:
        m = {"exported_note_ids": []}
    m.setdefault("exported_note_ids", [])
    # Notes exported while HiNotes was still transcribing (see
    # note_is_provisional) are recorded here instead of being treated as
    # permanently done, so a later run re-exports them once the real title
    # and AI summary exist.
    m.setdefault("provisional_note_ids", [])
    return m


def save_manifest(manifest: dict) -> None:
    with open(MANIFEST_PATH, "w") as f:
        json.dump(manifest, f, indent=2)


# A HiDock device upload lands in HiNotes BEFORE transcription/summarization
# finishes. In that window the note's title is still the raw device filename
# (e.g. "20260928-132429-Rec93.hda") and markdown/conciseSummary are null.
RAW_DEVICE_TITLE = re.compile(r"^\d{8}-\d{6}-Rec\d+\.hda$", re.IGNORECASE)


def note_is_provisional(title: str, info: dict) -> bool:
    """True when this note hasn't finished processing on HiNotes' side yet.

    WHY THIS EXISTS: the manifest used to record a note id as permanently
    exported the first time it was seen. Notes picked up mid-transcription
    were therefore frozen in the vault forever as a raw-filename title with
    "*(no summary available)*" as the body, and no rerun would ever fix them
    (found 2026-09-29: 7 notes affected, oldest from 09-09). Anything
    provisional must stay re-exportable.
    """
    if title and RAW_DEVICE_TITLE.match(title.strip()):
        return True
    if not (info.get("markdown") or info.get("conciseSummary")):
        return True
    return False


def build_summary_md(note_id: str, info: dict, folder_name: str) -> str:
    title = info.get("title") or f"Untitled {note_id}"
    create_time_ms = info.get("createTime")
    duration_ms = info.get("duration")
    tags_csv = info.get("tags") or ""
    tag_list = [t.strip() for t in tags_csv.split(",") if t.strip()]
    member_count = info.get("memberCount")
    source_device = info.get("sourceDevice") or "unknown"
    markdown_body = info.get("markdown") or info.get("conciseSummary") or "*(no summary available)*"

    fm_tags = ["type/note", "domain/project-delivery", "source/hinotes"]
    date_str = ""
    if create_time_ms:
        date_str = time.strftime("%Y-%m-%d", time.gmtime(int(create_time_ms) / 1000))

    frontmatter_lines = [
        "---",
        "tags:",
    ] + [f"  - {t}" for t in fm_tags] + [
        f'hinotes_note_id: "{note_id}"',
        f'hinotes_folder: "{folder_name}"',
        f'date: {date_str}' if date_str else "date:",
        f'duration_seconds: {int(duration_ms/1000) if duration_ms else 0}',
        f'member_count: {member_count if member_count is not None else "null"}',
        f'source_device: "{source_device}"',
        "---",
        "",
    ]
    body_lines = [f"# {title}", ""]
    if tag_list:
        body_lines.append("**Meeting tags:** " + ", ".join(tag_list))
        body_lines.append("")
    body_lines.append(markdown_body)
    body_lines.append("")
    return "\n".join(frontmatter_lines + body_lines)


def build_transcript_md(note_id: str, title: str, folder_name: str,
                         create_time_ms, segments: list) -> str:
    fm_tags = ["type/note", "domain/project-delivery", "source/hinotes"]
    date_str = ""
    if create_time_ms:
        date_str = time.strftime("%Y-%m-%d", time.gmtime(int(create_time_ms) / 1000))

    frontmatter_lines = [
        "---",
        "tags:",
    ] + [f"  - {t}" for t in fm_tags] + [
        f'hinotes_note_id: "{note_id}"',
        f'hinotes_folder: "{folder_name}"',
        f'date: {date_str}' if date_str else "date:",
        "---",
        "",
    ]
    body_lines = [f"# {title} — Transcript", ""]
    for seg in segments:
        speaker = seg.get("speaker") or "Unknown"
        begin = ms_to_hms(seg.get("beginTime"))
        sentence = (seg.get("sentence") or "").strip()
        if sentence:
            body_lines.append(f"**[{begin}] {speaker}:** {sentence}")
            body_lines.append("")
    return "\n".join(frontmatter_lines + body_lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--limit", type=int, default=None,
                         help="Cap how many NEW notes to export this run (for testing/batching)")
    args = parser.parse_args()

    token = os.environ.get("HINOTES_TOKEN")
    if not token:
        print("ERROR: set HINOTES_TOKEN env var")
        sys.exit(1)

    client = HiNotesClient(auth_token=token)
    if not client.is_token_valid():
        print("ERROR: HINOTES_TOKEN is invalid/expired. Ask the user to re-login and grab a fresh token.")
        sys.exit(1)

    manifest = load_manifest()
    already_exported = set(manifest["exported_note_ids"])
    provisional = set(manifest["provisional_note_ids"])
    # Provisional notes must be retried even though they're in
    # already_exported -- they were written as placeholders.
    print(f"Already exported: {len(already_exported)} notes "
          f"({len(provisional)} provisional/awaiting re-export)")

    # Build folder membership map (never hardcode folder count -- always
    # discover current folders live)
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

    # Full note list (paginate fully, no cap -- discover ALL notes,
    # including ones with no folder assignment)
    all_notes = client.list_all_notes(page_size=50)
    print(f"Total notes in account: {len(all_notes)}")

    new_notes = [n for n in all_notes
                 if n["id"] not in already_exported or n["id"] in provisional]
    n_retry = sum(1 for n in new_notes if n["id"] in provisional)
    print(f"New notes to export: {len(new_notes) - n_retry} "
          f"(+ {n_retry} provisional re-exports)")

    if args.limit:
        new_notes = new_notes[: args.limit]
        print(f"Limiting this run to {len(new_notes)} notes")

    exported_this_run = []
    errors = []

    for i, note in enumerate(new_notes):
        note_id = note["id"]
        title = note.get("title") or f"Untitled {note_id}"
        folder_name = note_to_folder.get(note_id, "Uncategorized")
        print(f"[{i+1}/{len(new_notes)}] {folder_name}/{title}")

        try:
            info = client.get_note_info(note_id)
        except Exception as e:
            print(f"  ERROR fetching note info: {e}")
            errors.append({"note_id": note_id, "title": title, "stage": "info", "error": str(e)})
            continue

        try:
            transcript = client.get_transcript(note_id)
        except Exception as e:
            print(f"  WARNING: transcript fetch failed: {e}")
            transcript = []

        safe_title = sanitize_filename(title)
        summ_dir = os.path.join(SUMM_ROOT, folder_name)
        trans_dir = os.path.join(TRANS_ROOT, folder_name)

        # Prefer the freshly-fetched title from note/info: the list endpoint
        # can still carry the stale raw device filename after processing has
        # completed.
        info_title = info.get("title") or title
        if info_title != title:
            title = info_title
            safe_title = sanitize_filename(title)

        is_provisional = note_is_provisional(title, info)
        if is_provisional:
            print("  NOTE still processing on HiNotes (no summary/raw title) — "
                  "writing placeholder and marking for re-export")

        summary_md = build_summary_md(note_id, info, folder_name)
        transcript_md = build_transcript_md(
            note_id, title, folder_name, note.get("createTime"), transcript
        )

        summ_path = os.path.join(summ_dir, f"{safe_title}.md")
        trans_path = os.path.join(trans_dir, f"{safe_title}.md")

        # If this note was previously written under a DIFFERENT (placeholder)
        # name or folder, remove those stale files so the real export doesn't
        # leave an orphaned "*(no summary available)*" duplicate behind.
        stale_removed = []
        if not is_provisional and note_id in provisional and not args.dry_run:
            for root, keep in ((SUMM_ROOT, summ_path), (TRANS_ROOT, trans_path)):
                for old in glob.glob(os.path.join(root, "*", "*.md")):
                    if os.path.abspath(old) == os.path.abspath(keep):
                        continue
                    try:
                        with open(old) as f:
                            head = f.read(600)
                    except OSError:
                        continue
                    if f'hinotes_note_id: "{note_id}"' in head:
                        os.remove(old)
                        stale_removed.append(old)
            for p in stale_removed:
                print(f"  removed stale placeholder: {p}")

        if args.dry_run:
            print(f"  [DRY RUN] would write:\n    {summ_path}\n    {trans_path}")
        else:
            os.makedirs(summ_dir, exist_ok=True)
            os.makedirs(trans_dir, exist_ok=True)
            with open(summ_path, "w") as f:
                f.write(summary_md)
            with open(trans_path, "w") as f:
                f.write(transcript_md)

        exported_this_run.append(note_id)
        if is_provisional:
            provisional.add(note_id)
        else:
            provisional.discard(note_id)

        # Update manifest incrementally so a crash mid-run doesn't lose
        # progress or cause re-export of already-written notes.
        if not args.dry_run:
            manifest["exported_note_ids"] = list(already_exported | set(exported_this_run))
            manifest["provisional_note_ids"] = sorted(provisional)
            save_manifest(manifest)

    print(f"\nDone. Exported {len(exported_this_run)} notes this run.")
    if errors:
        print(f"{len(errors)} errors encountered:")
        for e in errors:
            print(f"  {e['title']}: {e['error']}")
        with open(os.path.join(SCRIPT_DIR, "export_errors.json"), "w") as f:
            json.dump(errors, f, indent=2)


if __name__ == "__main__":
    main()
