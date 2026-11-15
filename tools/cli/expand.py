# morc: answer the cards' expand / shrink buttons at once while a session is live. Usage: morc expand watch
"""The expand button (fourth on the corner arc) only WAKES the agent ({ask: "expand"|"shrink"}); a session sees that wake on its next
prompt, so the button felt dead. Run `morc expand watch` in the background for the length of a session: it tails the inbox and does
the answer itself (docs/cards.md):
  expand: remember the card's spot and cap, set any `opts.expand_vars` its chip declares (e.g. video-card {"w": 1150}, so the content
          grows, not just the cap), then frac 0.92, centred, in front.
  shrink: put the vars back (null), restore the cap (a NEW cap unlocks the locked width so the card re-measures to the smaller content;
          patching the vars alone left a big empty card), then move it back to its spot."""
import json
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import HOME, RUNTIME  # noqa: E402

MORC = os.environ.get("MORC_BIN", "morc")
INBOX = os.path.join(RUNTIME, "inbox.jsonl")
ENV = dict(os.environ, MSYS_NO_PATHCONV="1")   # Git Bash would rewrite /vars/... into a Windows path


def morc(*a):
    r = subprocess.run([MORC, *a], capture_output=True, text=True, env=ENV)
    try:
        return json.loads(r.stdout.strip().splitlines()[-1])
    except (ValueError, IndexError):
        return {}


def expand_vars(chip):
    for base in (HOME, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")):
        try:
            with open(os.path.join(base, "cache", "components", chip + ".json"), encoding="utf-8-sig") as f:
                return (json.load(f).get("opts") or {}).get("expand_vars") or {}
        except (OSError, ValueError):
            continue
    return {}


def watch():
    before = {}   # card id -> (at [x, y], frac) from before its expand
    with open(INBOX, encoding="utf-8-sig") as f:
        f.seek(0, os.SEEK_END)
        print("expand watch: on (Ctrl+C stops)", flush=True)
        while True:
            line = f.readline()
            if not line:
                time.sleep(0.15)
                continue
            try:
                ev = json.loads(line)
            except ValueError:
                continue
            ask = (ev.get("payload") or {}).get("ask")
            cid = ev.get("surface")
            if ev.get("kind") != "wake" or ask not in ("expand", "shrink") or not cid:
                continue
            card = morc("card", cid, "get")
            ev_vars = expand_vars(card.get("chip") or "")
            if ask == "expand":
                before[cid] = ((card.get("rect") or [None, None])[:2], card.get("frac"))
                for k, v in ev_vars.items():
                    morc("patch", cid, "/vars/" + k, json.dumps(v))
                morc("card", cid, "set", json.dumps({"frac": 0.92, "center": True}))
                morc("card", cid, "front")
            else:
                at, frac = before.pop(cid, (None, None))
                for k in ev_vars:
                    morc("patch", cid, "/vars/" + k, "null")
                morc("card", cid, "size", "--frac", str(frac or 0.6))
                time.sleep(1.2)   # let it re-measure before it moves
                if at and at[0] is not None:
                    morc("card", cid, "move", "--at", f"{at[0]},{at[1]}")
                else:
                    morc("card", cid, "move", "--snap")
            print(f"{time.strftime('%H:%M:%S')} {ask} {cid}", flush=True)


if __name__ == "__main__":
    if sys.argv[1:2] == ["watch"]:
        try:
            watch()
        except KeyboardInterrupt:
            pass
        sys.exit(0)
    print(__doc__)
    sys.exit(2)
