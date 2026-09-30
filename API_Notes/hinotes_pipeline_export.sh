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
# This repo has MULTIPLE CONCURRENT WRITERS (Obsidian desktop app via
# obsidian-git, the "Sync Obsidian vault" cron job every 15min, the Trilium
# 2-way sync job, and this script every 30min). Use the shared lock+retry
# helper (git_sync_safe.sh) instead of a bare commit+push -- a bare
# pull-then-push is NOT safe under concurrent writers even if the pull
# happens first, because another writer can push in the gap between this
# script's pull and its own push (confirmed happening in practice 2026-09-05).
source "$HOME/.hermes/scripts/lib/git_sync_safe.sh"
SYNC_OUT=$(git_sync_safe "$VAULT_DIR" "$LABEL" "HiNotes pipeline: export new meetings")
SYNC_STATUS=$?
if [ $SYNC_STATUS -eq 2 ]; then
    git_sync_alert "$LABEL — MERGE CONFLICT" "git_sync_safe hit a merge conflict in $VAULT_DIR:\n\n$SYNC_OUT"
elif [ $SYNC_STATUS -ne 0 ]; then
    git_sync_alert "$LABEL — ERROR" "git_sync_safe failed in $VAULT_DIR:\n\n$SYNC_OUT"
else
    echo "Obsidian vault synced (pulled/committed/pushed as needed)."
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
