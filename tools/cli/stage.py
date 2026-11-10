# morc stage: register a prepared command so a card button can run it directly (zero tokens, no agent wakes)
#
#   morc stage <id> [--cwd DIR] -- <argv...>     add or replace actions.json entry <id> (marked staged, with the time)
#   morc stage --prune [--hours 24]              remove staged entries older than that (hand-written entries are never touched)
#   morc stage --list                            show the staged entries
#
# The agent prepares the exact command first (e.g. a commit with its message file already written), stages it, then shows a card whose
# button does {"do":"run","id":"<id>","into":"/out","then":{"/state":"done"},"fail":{"/state":"failed"}} (the confirm-run chip).
# The click runs it on the host and patches the result into the card: no model is involved. Never stage anything risky (push, delete).
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import HOME  # noqa: E402

PATH = os.path.join(HOME, "actions.json")


def load():
    try:
        with open(PATH, encoding="utf-8-sig") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {}


def save(d):
    with open(PATH, "w", encoding="utf-8") as f:
        json.dump(d, f, indent=2)
        f.write("\n")


args = sys.argv[1:]
d = load()
if not args or args[0] in ("-h", "--help"):
    print(__doc__ or open(__file__).read().split("import json")[0])
    sys.exit(0)
if args[0] == "--list":
    for k, v in d.items():
        if isinstance(v, dict) and v.get("_staged"):
            print(f"{k}: {' '.join(v.get('argv', []))}" + (f"  (cwd {v['cwd']})" if v.get("cwd") else ""))
    sys.exit(0)
if args[0] == "--prune":
    hours = float(args[args.index("--hours") + 1]) if "--hours" in args else 24.0
    cut = time.time() - hours * 3600
    gone = [k for k, v in d.items() if isinstance(v, dict) and v.get("_staged") and v["_staged"] < cut]
    for k in gone:
        del d[k]
    save(d)
    print(json.dumps({"pruned": gone}))
    sys.exit(0)
if "--" not in args or args.index("--") == len(args) - 1:
    sys.exit("usage: morc stage <id> [--cwd DIR] -- <argv...>")
head, argv = args[: args.index("--")], args[args.index("--") + 1 :]
sid = head[0]
if not sid.replace("-", "").replace("_", "").isalnum():
    sys.exit("id: letters, digits, - and _ only")
entry = {"argv": argv, "_staged": time.time()}
if "--cwd" in head:
    entry["cwd"] = os.path.abspath(head[head.index("--cwd") + 1])
d[sid] = entry
save(d)
print(json.dumps({"staged": sid, "argv": argv, "cwd": entry.get("cwd")}))
