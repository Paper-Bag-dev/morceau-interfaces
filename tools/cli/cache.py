# morc cache: keep the component cache lean with an LFU (least frequently used) policy that also expires cyclically.
#
#   morc cache stats [--all]            one line per entry: name, kind (comp|frag), uses, idle days, score, why-protected   (top 25 unless --all)
#   morc cache prune [--keep N] [--stale-days D] [--dry] [--if-due] [--quiet]
#   morc cache pin <name> | unpin <name>  never expire this one
#
# Score = uses * 0.5 ^ (idle days / 7): frequency with a one-week half-life, so a component that was popular once and then ignored
# fades by itself ("cyclic"). prune keeps the best --keep entries (default 24) and also removes anything idle longer than
# --stale-days (default 14), whatever its score. Never removed: pinned entries, the shipped seed library (cache/seed, re-creatable with
# `morc save cache/seed/<name>.json`), shipped fragments, and anything younger than 2 days. --if-due makes it a no-op unless 24 h passed
# since the last prune, so it is safe to call from any chore or hook. Counts come from the host (usage.json, bumped on every `use`) and
# from compose (usage_frag.json, bumped when a fragment is used). Looks in your data folder (HOME) and the install (APP).
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import APP, HOME, RUNTIME  # noqa: E402

LOC = RUNTIME
PINS = os.path.join(HOME, "prefs", "cache_pins.json")
STAMP = os.path.join(LOC, "cache_prune.stamp")
HALF_DAYS = 7.0
NOW = time.time()


def jload(p, d):
    try:
        return json.load(open(p, encoding="utf-8-sig"))
    except Exception:
        return d


def entries():
    seeds = set()
    for root in {HOME, APP}:
        sd = os.path.join(root, "cache", "seed")
        if os.path.isdir(sd):
            seeds |= {f[:-5] for f in os.listdir(sd)}
    pins = set(jload(PINS, []))
    use = jload(os.path.join(LOC, "usage.json"), {})
    fuse = jload(os.path.join(LOC, "usage_frag.json"), {})
    out, seen = [], set()
    for kind, folder, usage in (("comp", "components", use), ("frag", "fragments", fuse)):
        for root in (HOME, APP):
            d = os.path.join(root, "cache", folder)
            if not os.path.isdir(d):
                continue
            for f in os.listdir(d):
                path = os.path.join(d, f)
                if not f.endswith(".json") or f.startswith(".") or (kind, f) in seen:
                    continue
                seen.add((kind, f))
                name = f[:-5]
                u = usage.get(name, {})
                last = (u.get("last") or 0) / 1000 or os.path.getmtime(path)
                idle = max(0.0, (NOW - last) / 86400)
                uses = u.get("uses", 0)
                shipped = os.path.normcase(root) == os.path.normcase(APP) and os.path.normcase(HOME) != os.path.normcase(APP)
                why = "pinned" if name in pins else "shipped" if shipped else "seed" if (kind == "comp" and name in seeds) else "young" if (NOW - os.path.getmtime(path)) < 2 * 86400 else ""
                out.append({"name": name, "kind": kind, "uses": uses, "idle": round(idle, 1), "score": round(uses * 0.5 ** (idle / HALF_DAYS), 2), "protected": why, "path": path})
    return sorted(out, key=lambda e: -e["score"])


def arg(flag, default):
    a = sys.argv
    return type(default)(a[a.index(flag) + 1]) if flag in a else default


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "stats"
    if cmd == "stats":
        es = entries()
        for e in es if "--all" in sys.argv else es[:25]:
            print(f"{e['name']:<26} {e['kind']:<4} uses={e['uses']:<4} idle={e['idle']:<5}d score={e['score']:<6} {e['protected']}")
        print(f"{len(es)} entries")
    elif cmd == "prune":
        if "--if-due" in sys.argv and os.path.exists(STAMP) and NOW - os.path.getmtime(STAMP) < 86400:
            if "--quiet" not in sys.argv:
                print("not due")
            return
        keep, stale = arg("--keep", 24), arg("--stale-days", 14.0)
        es = entries()
        drop = [e for i, e in enumerate(es) if not e["protected"] and (i >= keep or e["idle"] > stale)]
        for e in drop:
            if "--dry" not in sys.argv:
                os.remove(e["path"])
        if "--dry" not in sys.argv:
            os.makedirs(LOC, exist_ok=True)
            open(STAMP, "w").write(str(NOW))
        if "--quiet" not in sys.argv:
            print(("would remove " if "--dry" in sys.argv else "removed ") + (", ".join(f"{e['name']}({e['kind']},uses={e['uses']},idle={e['idle']}d)" for e in drop) or "nothing") + f"; kept {len(es) - len(drop)}")
    elif cmd in ("pin", "unpin") and len(sys.argv) > 2:
        pins = set(jload(PINS, []))
        (pins.add if cmd == "pin" else pins.discard)(sys.argv[2])
        os.makedirs(os.path.dirname(PINS), exist_ok=True)
        json.dump(sorted(pins), open(PINS, "w", encoding="utf-8"))
        print("ok")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
