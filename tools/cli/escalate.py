# morc escalate: hand a task to the interactive Claude session instead of doing it (the background agent's way out)
#
#   morc escalate <ctx> --why "<one line: why I can't do it well>" [--task "<what the user asked>"]
#
# Writes {"kind":"escalate", payload:{ctx, why, task}} to the inbox. The session waits on the card's ctx, so this wakes it with the reason;
# its hook also lists it with the user's next prompt. Use it whenever docs/agent.md "Hand over to the session" says so.
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import RUNTIME  # noqa: E402

a = sys.argv[1:]
if not a or a[0].startswith("-"):
    sys.exit('usage: morc escalate <ctx> --why "..." [--task "..."]')
flag = lambda f: a[a.index(f) + 1] if f in a and a.index(f) + 1 < len(a) else ""
line = {"ts": int(time.time() * 1000), "surface": "", "topic": "agent.escalate", "kind": "escalate",
        "payload": {"ctx": a[0], "why": flag("--why"), "task": flag("--task")}}
os.makedirs(RUNTIME, exist_ok=True)
with open(os.path.join(RUNTIME, "inbox.jsonl"), "a", encoding="utf-8") as f:
    f.write(json.dumps(line) + "\n")
print(json.dumps({"escalated": a[0]}))
