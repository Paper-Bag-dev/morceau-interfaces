# morc: show ANY tracker JSON as live cards: a tickable checklist and/or a numbers strip, all described by the file itself
"""`morc tracker FILE [todo|strip|both] [--id ID]` reads FILE (a tracker JSON) and shows each view ONCE with `morc show`. FILE may be a path or a
name under `<MORC_HOME>/trackers` (e.g. `sprint`). The file carries its own card layout, so a new tracker needs no code:

  {"todo": [{"id","text","sub","badge","done",...}],            # the checklist rows (tick-<id> actions rewrite them)
   "progress": {"any_key": 3, ...},                              # free-form numbers the strip reads by pointer
   "views": {"todo":  {"title": "To do today", "id": "sprint-todo", "run": "tick-", "list": "/todo",
                       "show": {"region": "right", "ttl": 7200, "sticky": true}},
             "strip": {"id": "stat-strip", "cells": [{"label": "Lectures done", "value": "/progress/lectures_done",
                                                      "unit": "of {/progress/lectures_total}"}],   # up to 4; label/unit may use {/pointer}
                       "show": {"region": "above", "align": "left", "importance": 0.6, "ttl": 7200}}}}

The cards poll the file through `sources`, so they stay live. Placement is the host's job (region/align in the file); ladder: INDEX.md."""
import json
import os
import pathlib
import re
import subprocess
import sys

A2 = os.environ.get("MORC_BIN") or os.path.expandvars(r"%LOCALAPPDATA%\morc\bin\morc.exe")
HOME = pathlib.Path(os.environ.get("MORC_HOME") or pathlib.Path(__file__).resolve().parents[2])
SEED = HOME / "cache" / "seed" / "stat-strip.json"
CELLS = "abcd"


def find(arg):
    p = pathlib.Path(arg)
    if p.is_file():
        return p.resolve()
    root = HOME / "trackers"  # your data folder: trackers/<name>/live.json or trackers/<name>.json
    for cand in (root / arg / "live.json", root / f"{arg}.json"):
        if cand.is_file():
            return cand.resolve()
    sys.exit(f"tracker not found: {arg} (a path, or a name under {root})")


def get(obj, pointer):
    for part in pointer.strip("/").split("/"):
        obj = obj[int(part)] if isinstance(obj, list) else obj.get(part, "")
    return obj


def fill(text, doc):
    return re.sub(r"\{(/[^}]+)\}", lambda m: str(get(doc, m.group(1))), text)


def flags(show):
    out = []
    for k in ("region", "align", "importance", "ttl", "dx", "dy"):
        if k in show:
            out += [f"--{k}", str(show[k])]
    if show.get("sticky"):
        out.append("--sticky")
    return out


def push(spec, show):
    r = subprocess.run([A2, "show", "-"] + flags(show), input=json.dumps(spec), text=True, capture_output=True)
    print((r.stdout or r.stderr).strip())
    return r.returncode


def todo_spec(doc, view, src):
    ttl = int(view.get("show", {}).get("ttl", 7200))
    return {
        "id": view.get("id", "tracker-todo"),
        "opts": {"topic": view.get("topic", "tracker.todo"), "importance": "1.0", "min": [320, 260], "ttl": ttl},
        "components": [{"id": "root", "type": "checklist", "title": "{/title}", "items": {"bind": "/items"}, "run": view.get("run", "tick-")}],
        "data": {"title": view.get("title", doc.get("title", "To do")), "items": get(doc, view.get("list", "/todo"))},
        "sources": {"/items": src(view.get("list", "/todo"))},
    }


def strip_spec(doc, view, src):
    seed = json.loads(SEED.read_text(encoding="utf-8-sig"))
    ttl = int(view.get("show", {}).get("ttl", 7200))
    data, sources = {}, {}
    for key, cell in zip(CELLS, view["cells"][:4]):
        data[key] = {"label": fill(cell.get("label", ""), doc), "value": get(doc, cell["value"]), "unit": fill(cell.get("unit", ""), doc)}
        sources[f"/{key}/value"] = src(cell["value"])
        if cell.get("label", "").startswith("{/"):
            sources[f"/{key}/label"] = src(cell["label"].strip("{}"))
    return {
        "id": view.get("id", "tracker-strip"),
        "opts": dict(seed["opts"], topic=view.get("topic", "tracker.strip"), importance=str(view.get("show", {}).get("importance", 0.6)), ttl=ttl),
        "components": seed["components"], "data": data, "sources": sources,
    }


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit("usage: morc tracker FILE [todo|strip|both] [--id ID]")
    path = find(args[0])
    which = args[1] if len(args) > 1 and not args[1].startswith("--") else "both"
    doc = json.loads(path.read_text(encoding="utf-8-sig"))
    views = doc.get("views", {})

    def src(pointer, every=2):
        return {"file": str(path).replace("\\", "/"), "pointer": pointer, "every": every}

    rc = 0
    for name, build in (("strip", strip_spec), ("todo", todo_spec)):
        if which in (name, "both") and name in views:
            view = dict(views[name])
            if "--id" in args and name == "todo":
                view["id"] = args[args.index("--id") + 1]
            rc |= push(build(doc, view, src), view.get("show", {}))
    sys.exit(rc)


if __name__ == "__main__":
    main()
