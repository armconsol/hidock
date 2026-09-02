#!/bin/bash
# HiNotes pipeline -- step 2/2 (mechanical, no reasoning):
# Export any new/unexported meetings to Google Drive (summaries + transcripts
# + recordings) and to the Obsidian vault (summaries + transcripts, NO audio),
# tag the new Obsidian files, and commit+push both repos.
#
# Must be run AFTER classification (folder assignment + template regen for
# any newly-Uncategorized notes) is already done -- see check_new_meetings.py
# and the cron job prompt for that reasoning step.
#
# Idempotent: both export_to_drive.py and export_to_obsidian.py track
# already-exported note IDs in their own manifest JSON files and skip
# anything already done, so safe to run every tick even if nothing is new.
#
# Usage: HINOTES_TOKEN=<token> bash hinotes_pipeline_export.sh
set -uo pipefail

HINOTES_DIR="/home/sarman/repos/hinotes"
API_DIR="$HINOTES_DIR/API_Notes"
VAULT_DIR="/home/sarman/repos/obsidian"
LABEL="HiNotes pipeline"
source "$HOME/.hermes/scripts/lib/git_sync_alert.sh"

if [ -z "${HINOTES_TOKEN:-}" ]; then
    # Fall back to .env if not already exported by the caller
    if [ -f "$HINOTES_DIR/.env" ]; then
        export HINOTES_TOKEN=$(grep -E '^HINOTES_TOKEN=' "$HINOTES_DIR/.env" | tail -1 | cut -d'=' -f2-)
    fi
fi
if [ -z "${HINOTES_TOKEN:-}" ]; then
    git_sync_alert "$LABEL — ERROR" "HINOTES_TOKEN not set and not found in $HINOTES_DIR/.env"
    echo "ERROR: HINOTES_TOKEN not set"
    exit 1
fi

cd "$API_DIR" || exit 1

echo "=== Drive export (summaries + transcripts + recordings) ==="
DRIVE_OUT=$(python3 export_to_drive.py 2>&1)
DRIVE_STATUS=$?
echo "$DRIVE_OUT"
if [ $DRIVE_STATUS -ne 0 ]; then
    git_sync_alert "$LABEL — ERROR" "export_to_drive.py failed (exit $DRIVE_STATUS):\n\n$DRIVE_OUT"
fi

echo "=== Obsidian export (summaries + transcripts, no audio) ==="
OBS_OUT=$(python3 export_to_obsidian.py 2>&1)
OBS_STATUS=$?
echo "$OBS_OUT"
if [ $OBS_STATUS -ne 0 ]; then
    git_sync_alert "$LABEL — ERROR" "export_to_obsidian.py failed (exit $OBS_STATUS):\n\n$OBS_OUT"
fi

# --- Tag any newly-written Obsidian files (idempotent, only rewrites files
#     whose tags would change) ---
if [ "$OBS_STATUS" -eq 0 ]; then
    echo "=== Tagging vault ==="
    TAG_OUT=$(python3 ~/.local/lib/obsidian-tagger/apply_tags.py "$VAULT_DIR" --apply 2>&1)
    echo "$TAG_OUT"
fi

# --- Commit+push the Obsidian vault (new Meetings/Summeries|Transcripts files) ---
cd "$VAULT_DIR" || exit 1
if [ -n "$(git status --porcelain)" ]; then
    git add -A
    TS=$(date '+%Y-%m-%d %H:%M:%S')
    if git commit -m "HiNotes pipeline: export new meetings (${TS})" >/dev/null 2>&1; then
        PUSH_OUT=$(git push origin main 2>&1)
        if [ $? -ne 0 ]; then
            git_sync_alert "$LABEL — ERROR" "obsidian vault commit ok but push failed:\n\n$PUSH_OUT"
        else
            echo "Committed and pushed new Obsidian export files."
        fi
    else
        git_sync_alert "$LABEL — ERROR" "obsidian vault git commit failed"
    fi
fi

# --- Commit+push the hinotes repo's manifest state (idempotency tracking) ---
cd "$API_DIR" || exit 1
if [ -n "$(git status --porcelain drive_export_manifest.json export_manifest.json 2>/dev/null)" ]; then
    git add drive_export_manifest.json export_manifest.json drive_export_errors.json export_errors.json 2>/dev/null
    TS=$(date '+%Y-%m-%d %H:%M:%S')
    if git commit -m "HiNotes pipeline: update export manifests (${TS})" >/dev/null 2>&1; then
        PUSH_OUT=$(git push origin main 2>&1)
        if [ $? -ne 0 ]; then
            git_sync_alert "$LABEL — ERROR" "hinotes repo manifest commit ok but push failed:\n\n$PUSH_OUT"
        fi
    fi
fi

echo "=== Pipeline export step complete ==="
