# morc wake: the chore agent sets its OWN next wake (flags in prefs/wake_flags.json, read by the host every 10 s, costs no tokens).
#
#   morc wake set <name> [--after MIN] [--activity N] [--when /pointer:op:value] [--source FILE] [--repeat MIN] [--reason "..."]
#   morc wake list
#   morc wake clear <name> | --all
#
# A flag is DUE when ALL its conditions hold: --after (minutes from now) has passed, --activity (that many window changes have been
# logged since it was set: "enough data has accumulated") and --when (a number in a JSON file you point at with --source FILE or MORC_STATS_FILE, e.g. /today/video_min:gte:100; read from
# that file). When due, the host asks for a chore wake, which still passes the gates (live session, PC idle,
# rate limit, chore cooldown); if gated it stays due and is retried. One-shot unless --repeat MIN. Setting a flag with an existing name
# replaces it. Example before sleeping:
#   morc wake set check-in --after 45 --activity 12 --reason "stats accumulated, show a nudge"
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import HOME  # noqa: E402

FILE = os.path.join(HOME, "prefs", "wake_flags.json")
SOURCE = os.environ.get("MORC_STATS_FILE", "")   # the JSON file --when reads numbers from (or pass --source FILE)


def load():
    try:
        return json.load(open(FILE, encoding="utf-8"))
    except Exception:
        return []


def save(flags):
    os.makedirs(os.path.dirname(FILE), exist_ok=True)
    json.dump(flags, open(FILE, "w", encoding="utf-8"), indent=2)


def opt(a, name, cast=None, default=None):
    if name not in a:
        return default
    v = a[a.index(name) + 1]
    return cast(v) if cast else v


def main():
    a = sys.argv[1:]
    cmd = a[0] if a else "list"
    now = int(time.time() * 1000)
    if cmd == "set" and len(a) > 1:
        name = a[1]
        f = {"name": name, "since_ts": now, "after_ts": now + int(float(opt(a, "--after", float, 0)) * 60000), "reason": opt(a, "--reason", None, name)}
        if "--activity" in a:
            f["min_activity"] = opt(a, "--activity", int)
        if "--repeat" in a:
            f["repeat_min"] = opt(a, "--repeat", float)
        if "--when" in a:
            ptr, op, val = (opt(a, "--when") or "").rsplit(":", 2)
            src = opt(a, "--source", None, SOURCE)
            if not src:
                sys.exit("--when needs a JSON file to read: pass --source FILE or set MORC_STATS_FILE")
            f["when"] = {"file": src, "pointer": ptr, "" + op: float(val)}
        flags = [x for x in load() if x.get("name") != name] + [f]
        save(flags)
        print(json.dumps({"ok": True, "flags": len(flags)}))
    elif cmd == "list":
        for f in load():
            due_in = max(0, (f.get("after_ts", 0) - now) // 60000)
            print(f"{f['name']:<18} in {due_in}m  activity>={f.get('min_activity', 0)}  when={json.dumps(f.get('when', {}).get('pointer', ''))}  repeat={f.get('repeat_min', '-')}  {f.get('reason', '')}")
        if not load():
            print("(no flags: the default heartbeat rule in triggers/ still applies)")
    elif cmd == "clear":
        keep = [] if "--all" in a else [x for x in load() if x.get("name") != (a[1] if len(a) > 1 else "")]
        save(keep)
        print("ok")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
