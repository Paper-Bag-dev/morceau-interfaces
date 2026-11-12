# morc init: set up your data folder and the shipped components in one go. Safe to run again.
#
#   morc init                 create the data folder (default: the folder `morc` already uses), seed it, start the host
#   morc init --home DIR      keep your data in DIR instead (remembered in <runtime>/home.txt, so no environment variable is needed)
#   morc init --no-seed       only create folders and starter files, do not cache the shipped components
#
# What it does: makes DIR/{prefs,triggers,cache/components,cache/recipes,cache/fragments,tools/cli}, copies examples/actions.example.json to
# actions.json if you have none, saves every shipped seed (ask-permission, bike-viewer, graph-view, week-chart) so `morc list` works, and prints
# the next steps. It never overwrites a file you already have.
import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import APP, HOME as CURRENT_HOME, RUNTIME, BIN  # noqa: E402


def main():
    a = sys.argv[1:]
    home = os.path.abspath(a[a.index("--home") + 1]) if "--home" in a else CURRENT_HOME
    os.makedirs(RUNTIME, exist_ok=True)
    if "--home" in a:
        open(os.path.join(RUNTIME, "home.txt"), "w", encoding="utf-8").write(home)
    for d in ("prefs", "triggers", os.path.join("cache", "components"), os.path.join("cache", "recipes"), os.path.join("cache", "fragments"), os.path.join("tools", "cli")):
        os.makedirs(os.path.join(home, d), exist_ok=True)
    made = []
    src = os.path.join(APP, "examples", "actions.example.json")
    dst = os.path.join(home, "actions.json")
    if os.path.exists(src) and not os.path.exists(dst):
        shutil.copyfile(src, dst)
        made.append("actions.json")
    saved = []
    if "--no-seed" not in a:
        seeds = os.path.join(APP, "cache", "seed")
        for f in sorted(os.listdir(seeds)) if os.path.isdir(seeds) else []:
            if f.endswith(".json"):
                r = subprocess.run([BIN, "save", os.path.join(seeds, f)], capture_output=True, text=True, timeout=120)
                if '"ok":true' in r.stdout.replace(" ", ""):
                    saved.append(f[:-5])
                else:
                    # one failure means the host is not reachable, so every other seed would fail the same way: stop and say why once
                    why = (r.stderr or r.stdout).strip().replace("morc: ", "")[:400]
                    print(f"morc init stopped: could not save the shipped components because {why}", file=sys.stderr)
                    print(f"Your data folder is ready at {home}. Fix the above, then run `morc init` again (it is safe to repeat).", file=sys.stderr)
                    sys.exit(1)
        rec = os.path.join(APP, "cache", "recipes")
        for f in sorted(os.listdir(rec)) if os.path.isdir(rec) else []:     # shipped recipes become cached components too (compose without --use only saves)
            if f.endswith(".json"):
                r = subprocess.run([sys.executable, os.path.join(os.path.dirname(os.path.abspath(__file__)), "compose.py"), "--recipe", os.path.join(rec, f)], capture_output=True, text=True, timeout=120, env=dict(os.environ, MORC_HOME=home))
                if '"name"' in r.stdout:
                    saved.append(json.loads(r.stdout)["name"])
    print(json.dumps({"app": APP, "home": home, "created": made, "saved": saved}, separators=(",", ":")))
    print("next: `morc use ask-permission` to see a widget, `morc list` for the cache, `morc commands` for the tools; docs/README.md has the map.")


if __name__ == "__main__":
    main()
