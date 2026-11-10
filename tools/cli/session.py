# morc session: mark the interactive Claude session as live right now (what the Claude Code hooks do on every prompt)
#
#   morc session        touch it; in "no_session" mode the background agent then leaves the session's answers alone for session_fresh_s
#
# A session waiting on a card's answer runs this in its wait loop, so the background agent never answers in its place.
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import RUNTIME  # noqa: E402

if "MORC_HEADLESS" in os.environ:   # the background agent itself must never count as the session
    sys.exit(0)
os.makedirs(RUNTIME, exist_ok=True)
with open(os.path.join(RUNTIME, "session.seen"), "w", encoding="utf-8") as f:
    f.write(str(int(time.time() * 1000)))
