"""
One-time (or manually re-run) historical backfill for the Ryzyliant
domain-knowledge learning pass over ALL existing HiNotes meeting summaries.

The daily Morning Briefing cron job only processes summaries newer than
`~/.hermes/state/hinotes_learning_last_run.marker` (incremental — cheap,
a handful of meetings per day). That marker was seeded at the point this
backfill script/pipeline was introduced, so the ~330 pre-existing meetings
would otherwise NEVER get reviewed for domain knowledge by the daily job.

This script does NOT do the LLM reasoning itself (extracting pain points/
terminology needs judgment, not scripting) -- it just prepares a batched,
resumable manifest of NOT-YET-REVIEWED summaries so an agent (this session,
or a dispatched subagent) can work through them in controlled chunks
without trying to read 330 files in one context window.

Usage:
    python3 hinotes_backfill_manifest.py                # show status
    python3 hinotes_backfill_manifest.py --next 20       # print next 20 unreviewed paths
    python3 hinotes_backfill_manifest.py --mark-reviewed <path> [<path> ...]
"""
import sys
import os
import json
import argparse

VAULT_SUMM_ROOT = "/home/sarman/repos/obsidian/Meetings/Summeries"
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MANIFEST_PATH = os.path.join(SCRIPT_DIR, "domain_learning_backfill_manifest.json")


def all_summary_paths():
    paths = []
    for root, _, files in os.walk(VAULT_SUMM_ROOT):
        for f in files:
            if f.endswith(".md"):
                paths.append(os.path.join(root, f))
    return sorted(paths)


def load_manifest():
    if os.path.exists(MANIFEST_PATH):
        with open(MANIFEST_PATH) as f:
            return json.load(f)
    return {"reviewed_paths": []}


def save_manifest(m):
    with open(MANIFEST_PATH, "w") as f:
        json.dump(m, f, indent=2)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--next", type=int, default=None,
                         help="Print the next N not-yet-reviewed summary file paths")
    parser.add_argument("--mark-reviewed", nargs="+", default=None,
                         help="Mark one or more paths as reviewed")
    args = parser.parse_args()

    manifest = load_manifest()
    reviewed = set(manifest["reviewed_paths"])

    if args.mark_reviewed:
        for p in args.mark_reviewed:
            reviewed.add(p)
        manifest["reviewed_paths"] = sorted(reviewed)
        save_manifest(manifest)
        print(f"Marked {len(args.mark_reviewed)} path(s) reviewed. "
              f"Total reviewed: {len(reviewed)}")
        return

    total = all_summary_paths()
    unreviewed = [p for p in total if p not in reviewed]

    if args.next:
        for p in unreviewed[: args.next]:
            print(p)
        return

    print(f"Total meeting summaries: {len(total)}")
    print(f"Reviewed for domain knowledge: {len(reviewed)}")
    print(f"Remaining: {len(unreviewed)}")


if __name__ == "__main__":
    main()
