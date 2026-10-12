#!/usr/bin/env python3
"""Rotate HINOTES_TOKEN in ~/repos/hinotes/.env safely.

Why this exists: there is no headless way to obtain a HiNotes token (the
login is Google reCAPTCHA-gated, enforced server-side). Rotation is always
manual, and the obvious manual routes all leak the secret:

  - `HINOTES_TOKEN=... cmd` or `$(...)` puts it in `ps` argv
  - pasting it into a chat/agent session puts it in conversation logs
  - a `.env.bak` copy can hold the OLD live token in a stageable file

So: read the token from a hidden getpass prompt, validate it BEFORE
touching anything on disk, then rewrite only the one line, in place,
preserving every other key and mode 0600. No backup file is written on
purpose (see .gitignore note in the hinotes-api skill).

Usage:  python3 rotate_token.py            # prompts, validates, writes
        python3 rotate_token.py --check    # just report current token state
"""

import argparse
import os
import re
import stat
import sys
from datetime import date
from getpass import getpass

ENV_PATH = os.path.expanduser("~/repos/hinotes/.env")
CLIENT_DIR = os.path.expanduser("~/repos/hinotes/API_Notes")

sys.path.insert(0, CLIENT_DIR)


def read_env(path):
    """Return (lines, {key: value}) preserving original line order/formatting."""
    with open(path) as fh:
        lines = fh.read().splitlines()
    values = {}
    for line in lines:
        if line.strip().startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        values[k.strip()] = v.strip()
    return lines, values


def validate(token):
    """True only if folder/list accepts the token.

    Deliberately NOT /v1/user/info: with a dead token that endpoint silently
    auto-provisions an anonymous Guest account and returns error:0, so it
    reports success for a token that cannot read any of the user's data.
    """
    from hinotes_client import HiNotesClient

    return HiNotesClient(auth_token=token).is_token_valid()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true",
                    help="report current token validity and exit")
    args = ap.parse_args()

    if not os.path.exists(ENV_PATH):
        sys.exit(f"missing {ENV_PATH}")

    lines, values = read_env(ENV_PATH)
    current = values.get("HINOTES_TOKEN")

    if args.check:
        if not current:
            sys.exit("no HINOTES_TOKEN set")
        ok = validate(current)
        print(f"obtained: {values.get('HINOTES_TOKEN_OBTAINED', 'unknown')}")
        print(f"length:   {len(current)}")
        print(f"valid:    {ok}")
        sys.exit(0 if ok else 1)

    print("Paste the new accessToken (input hidden; nothing is echoed).")
    print("Get it via: hinotes.hidock.com/login -> DevTools console ->")
    print("  localStorage.getItem('accessToken')")
    new = getpass("token: ").strip().strip("'\"")

    if not new:
        sys.exit("aborted: empty input")
    if new == current:
        sys.exit("aborted: that is already the token on disk")
    if re.search(r"\s", new):
        sys.exit("aborted: token contains whitespace -- bad paste?")

    # Validate BEFORE writing, so a bad paste can never replace a working token.
    print("validating against folder/list ...")
    if not validate(new):
        sys.exit("REJECTED: token failed validation; .env left untouched")
    print("validated OK")

    today = date.today().isoformat()
    out, saw_token, saw_date = [], False, False
    for line in lines:
        if line.startswith("HINOTES_TOKEN="):
            out.append(f"HINOTES_TOKEN={new}")
            saw_token = True
        elif line.startswith("HINOTES_TOKEN_OBTAINED="):
            out.append(f"HINOTES_TOKEN_OBTAINED={today}")
            saw_date = True
        else:
            out.append(line)
    if not saw_token:
        out.append(f"HINOTES_TOKEN={new}")
    if not saw_date:
        out.append(f"HINOTES_TOKEN_OBTAINED={today}")

    # Write via a 0600 temp file in the same dir, then atomic replace, so the
    # secret is never world-readable even momentarily and a crash mid-write
    # cannot truncate the live .env.
    tmp = ENV_PATH + ".tmp"
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as fh:
        fh.write("\n".join(out) + "\n")
    os.replace(tmp, ENV_PATH)
    os.chmod(ENV_PATH, stat.S_IRUSR | stat.S_IWUSR)

    print(f"wrote {ENV_PATH} (mode 0600), HINOTES_TOKEN_OBTAINED={today}")
    print("other keys preserved; no backup file written (a .bak would hold "
          "the old live token)")


if __name__ == "__main__":
    main()
