# Shared by the CLI plugins (a leading underscore keeps it out of `morc commands`). It mirrors src/paths.rs so the Rust host and the tools agree on where things live:
#   APP     the install: shipped UI, tools, docs and generic components            (MORC_APP, else two folders above this file)
#   HOME    your data: prefs/, triggers/, actions.json, cache/, your own recipes   (MORC_HOME, else <runtime>/home.txt, else APP)
#   RUNTIME per-machine state: token, logs, usage counts, the installed exe        (%LOCALAPPDATA%\morc; macOS: ~/Library/Application Support/morc)
#   BIN     the morc executable                                                    (MORC_BIN, else <runtime>/bin/morc.exe, else APP/bin/morc.exe; no .exe on macOS)
import os
import sys

APP = os.environ.get("MORC_APP") or os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
if sys.platform == "darwin":
    RUNTIME = os.path.join(os.path.expanduser("~"), "Library", "Application Support", "morc")
else:
    RUNTIME = os.path.join(os.environ.get("LOCALAPPDATA") or os.path.expanduser("~"), "morc")
EXE = "morc.exe" if os.name == "nt" else "morc"


def _home():
    h = os.environ.get("MORC_HOME")
    if h:
        return h
    try:
        t = open(os.path.join(RUNTIME, "home.txt"), encoding="utf-8-sig").read().strip()
        if t:
            return t
    except OSError:
        pass
    return APP


HOME = _home()


def _bin():
    for p in (os.environ.get("MORC_BIN"), os.path.join(RUNTIME, "bin", EXE), os.path.join(APP, "bin", EXE), os.path.join(APP, "target", "release", EXE)):
        if p and os.path.exists(p):
            return p
    return "morc"          # fall back to PATH


BIN = _bin()


def find(*rel):
    """First existing path among HOME/<rel> and APP/<rel> (your data shadows the shipped one); HOME/<rel> if neither exists yet."""
    for root in (HOME, APP):
        p = os.path.join(root, *rel)
        if os.path.exists(p):
            return p
    return os.path.join(HOME, *rel)
