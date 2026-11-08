# morc place: read and change the placement policy (prefs/placement.json) from the command line. The host hot-reloads the file,
# so nothing here ever needs a rebuild or a restart.
#
#   morc place show                              summary: weights, thresholds, anchors
#   morc place get <path>                        e.g. weights.toast   anchors.below
#   morc place set <path> <json> [--why "..."]   e.g. set weights.toast_explicit 0.5   set auto_move.cooldown_s 20
#   morc place anchor <name> '<json>' [--why ..] define or replace an anchor, e.g.
#        morc place anchor beside '{"terms":[{"edge":"left","to":"fg.right","w":3,"off":6},{"edge":"top","to":"fg.top","w":1.2}]}'
#   morc place reset                             delete the file; the host writes the defaults again
# Every change is appended to prefs/placement_log.md (who/why), like the annoyance vector.
import datetime
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import HOME  # noqa: E402

FILE = os.path.join(HOME, "prefs", "placement.json")
LOG = os.path.join(HOME, "prefs", "placement_log.md")


def load():
    if not os.path.exists(FILE):
        sys.exit("no placement.json yet: run any `morc use ...` once and the host writes the defaults")
    return json.load(open(FILE, encoding="utf-8"))


def save(d, note):
    json.dump(d, open(FILE, "w", encoding="utf-8"), indent=2)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(f"- {datetime.datetime.now():%Y-%m-%d %H:%M} {note}\n")


def walk(d, path, create=False):
    keys = path.split(".")
    for k in keys[:-1]:
        if k not in d:
            if not create:
                sys.exit(f"no such key: {path}")
            d[k] = {}
        d = d[k]
    return d, keys[-1]


def flag(name):
    a = sys.argv
    return a[a.index(name) + 1] if name in a else ""


def main():
    a = [x for x in sys.argv[1:]]
    cmd = a[0] if a else "show"
    why = flag("--why")
    if "--why" in a:
        i = a.index("--why")
        a = a[:i] + a[i + 2:]
    if cmd == "show":
        d = load()
        print(json.dumps({"weights": d.get("weights"), "auto_move": d.get("auto_move"), "margin": d.get("margin"), "gap": d.get("gap"), "toast_zone": d.get("toast_zone"), "explicit_anchors": d.get("explicit_anchors"), "anchors": sorted(d.get("anchors", {}))}, separators=(",", ":")))
    elif cmd == "get" and len(a) > 1:
        d = load()
        parent, k = walk(d, a[1])
        print(json.dumps(parent.get(k), separators=(",", ":")))
    elif cmd == "set" and len(a) > 2:
        d = load()
        parent, k = walk(d, a[1], create=True)
        old = parent.get(k)
        parent[k] = json.loads(a[2])
        save(d, f"set {a[1]} {json.dumps(old)} -> {a[2]}" + (f" ({why})" if why else ""))
        print("ok")
    elif cmd == "anchor" and len(a) > 2:
        d = load()
        d.setdefault("anchors", {})[a[1]] = json.loads(a[2])
        save(d, f"anchor {a[1]} = {a[2]}" + (f" ({why})" if why else ""))
        print("ok")
    elif cmd == "reset":
        if os.path.exists(FILE):
            os.remove(FILE)
        with open(LOG, "a", encoding="utf-8") as f:
            f.write(f"- {datetime.datetime.now():%Y-%m-%d %H:%M} reset to defaults\n")
        print("ok (the host rewrites the defaults on the next push)")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
