# morc compose: build a widget from small reusable FRAGMENTS and cache the result, so the same shape is never built twice.
#
#   morc compose --recipe recipe.json [--use] [-- <extra morc use flags>]
#   morc compose '<recipe json>' [--use]
#
# recipe: {"name": "optional", "desc": "...", "topic": "stats.x", "opts": {"min": [w,h], "ttl": 20},
#          "layout": {"type": "column", "gap": 6}                                  (default: a column of the parts, in order)
#                  | {"type": "wing", "main": ["a"], "aside": ["b", "c"]},          (glanceable part beside, the rest tucked below)
#          "parts": [{"id": "a", "frag": "stat-pair"}, {"id": "b", "frag": "note-list"}],
#          "data":  {"a": {"l1": "Screen today", "v1": 3.4, "u1": "h", ...}, "b": {"title": "...", "items": [...]}}}
# Fragments live in cache/fragments/*.json; their data paths are written /@/field and become /<part id>/field.
# The STRUCTURE (components + opts, never the data) is hashed: an identical structure is a cache hit (no save). Without "name" it is
# called c-<hash>. With --use the widget is shown straight away (pass placement flags after `--`, e.g. -- --region right).
# Fragment use is counted for the LFU expiry (morc cache). Output is one short JSON line.
import hashlib
import json
import re
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import APP, HOME, RUNTIME, BIN, find  # noqa: E402

A2 = BIN
ROOT = HOME
FRAG_DIRS = [os.path.join(HOME, "cache", "fragments"), os.path.join(APP, "cache", "fragments")]   # your fragments shadow the shipped ones
COMPS = os.path.join(HOME, "cache", "components")
FRAG_USAGE = os.path.join(RUNTIME, "usage_frag.json")


def frag_file(name):
    return next((p for p in (os.path.join(d, name + ".json") for d in FRAG_DIRS) if os.path.exists(p)), os.path.join(FRAG_DIRS[0], name + ".json"))


PH = re.compile(r"\{\{\s*([\w.\-]+)\s*\}\}")


def lookup(scope, key):
    cur = scope
    for k in key.split("."):
        if isinstance(cur, dict) and k in cur:
            cur = cur[k]
        else:
            raise KeyError(key)
    return cur


def subst(node, scope):
    """Replace {{name}} / {{var.field}} in any JSON value. A string that is ONLY a placeholder keeps the value's type (number, bool, list)."""
    if isinstance(node, str):
        m = PH.fullmatch(node.strip())
        if m:
            return lookup(scope, m.group(1))
        return PH.sub(lambda mm: str(lookup(scope, mm.group(1))), node)
    if isinstance(node, list):
        return [subst(x, scope) for x in node]
    if isinstance(node, dict):
        return {k: subst(v, scope) for k, v in node.items()}
    return node


def load_vars(spec):
    """recipe "vars": {"sf": "ui/models/assets/<uid>/attribution.json", "x": {...inline...}} -> {{sf.src}}, {{x.y}}"""
    out = {}
    for k, v in (spec or {}).items():
        out[k] = json.load(open(find(v), encoding="utf-8-sig")) if isinstance(v, str) else v
    return out


def flags(argv, name):
    return [argv[i + 1] for i, a in enumerate(argv) if a == name and i + 1 < len(argv)]


def parse(v):
    try:
        return json.loads(v)
    except Exception:
        return v


def bump(names):
    try:
        u = json.load(open(FRAG_USAGE, encoding="utf-8"))
    except Exception:
        u = {}
    for n in names:
        e = u.get(n, {"uses": 0})
        u[n] = {"uses": e.get("uses", 0) + 1, "last": int(time.time() * 1000)}
    os.makedirs(os.path.dirname(FRAG_USAGE), exist_ok=True)
    json.dump(u, open(FRAG_USAGE, "w", encoding="utf-8"))


def expand(part, scope=None):
    pid = part["id"]
    path = frag_file(part["frag"])
    if not os.path.exists(path):
        sys.exit(f"no fragment '{part['frag']}' (have: {', '.join(sorted({f[:-5] for d in FRAG_DIRS if os.path.isdir(d) for f in os.listdir(d)}))})")
    fr = json.load(open(path, encoding="utf-8"))
    params = {**fr.get("defaults", {}), **subst(part.get("with", {}), scope or {})}     # fragment defaults, then this part's own values
    fr = subst(fr, {**(scope or {}), **params})                                        # {{param}} -> value; /@/ data paths are handled below
    ids = {c["id"]: f"{pid}_{c['id']}" for c in fr["components"]}
    out = []
    for c in fr["components"]:
        c = json.loads(json.dumps(c).replace("/@/", f"/{pid}/"))
        c["id"] = ids[c["id"]]
        if "children" in c:
            c["children"] = [ids.get(k, k) for k in c["children"]]
        out.append(c)
    return ids[fr["root"]], out


def main():
    argv = sys.argv[1:]
    extra = []
    if "--" in argv:
        i = argv.index("--")
        argv, extra = argv[:i], argv[i + 1:]
    show = "--use" in argv
    argv = [a for a in argv if a != "--use"]
    if "--recipe" in argv:
        recipe = json.load(open(argv[argv.index("--recipe") + 1], encoding="utf-8-sig"))   # tolerate a BOM (PowerShell writes one)
    elif argv:
        recipe = json.loads(argv[0])
    else:
        sys.exit(__doc__)

    scope = load_vars(recipe.get("vars"))
    for kv in flags(argv, "--set"):        # --set part.param=value : override a fragment parameter for this push (no recipe edit)
        k, v = kv.split("=", 1)
        pid, key = k.split(".", 1)
        for p in recipe["parts"]:
            if p["id"] == pid:
                p.setdefault("with", {})[key] = parse(v)
    for kv in flags(argv, "--data"):       # --data part.field=value : override a data value for this push
        k, v = kv.split("=", 1)
        pid, key = k.split(".", 1)
        recipe.setdefault("data", {}).setdefault(pid, {})[key] = parse(v)

    # Completeness gate: never build or show a widget with a missing label / value / unit. Fragments declare "required" fields.
    data0 = recipe.get("data", {})
    problems = []
    for part in recipe["parts"]:
        need = json.load(open(frag_file(part["frag"]), encoding="utf-8")).get("required", [])
        have = data0.get(part["id"], {}) if isinstance(data0.get(part["id"], {}), dict) else {}
        for k in need:
            v = have.get(k)
            if v is None or v == "" or v == [] or (isinstance(v, str) and v.strip() in ("-", "—", "?", "n/a", "N/A")):
                problems.append(f"{part['id']}.{k}")
        if "/@/unit" in open(frag_file(part["frag"]), encoding="utf-8").read() and not have.get("unit"):
            problems.append(f"{part['id']}.unit (every number needs its unit: min, h, %, ...)")
    if problems:
        sys.exit("incomplete, not pushed. Missing: " + ", ".join(sorted(set(problems))))

    roots, comps = {}, []
    for part in recipe["parts"]:
        r, cs = expand(part, scope)
        roots[part["id"]] = r
        comps += cs
    lay = recipe.get("layout") or {"type": "column", "gap": 6}
    if lay.get("type") == "wing":
        root = {"id": "root", "type": "wing", "main": [roots[p] for p in lay.get("main", [])], "aside": [roots[p] for p in lay.get("aside", [])]}
    else:
        root = {"id": "root", "type": lay.get("type", "column"), "gap": lay.get("gap", 6), "children": [roots[p["id"]] for p in recipe["parts"]]}
        if lay.get("align"):
            root["align"] = lay["align"]
    comps = [root] + comps
    opts = dict(recipe.get("opts", {}))
    if recipe.get("topic"):
        opts["topic"] = recipe["topic"]
    data = recipe.get("data", {})
    structure = json.dumps({"c": comps, "o": opts}, sort_keys=True)
    name = recipe.get("name") or "c-" + hashlib.sha1(structure.encode()).hexdigest()[:8]
    sh = hashlib.sha1(structure.encode()).hexdigest()
    hfile = os.path.join(RUNTIME, "compose_hashes.json")
    try:
        hashes = json.load(open(hfile, encoding="utf-8"))
    except Exception:
        hashes = {}
    status = "hit"
    if not os.path.exists(os.path.join(COMPS, name + ".json")) or hashes.get(name) != sh:   # a changed structure re-saves, even under the same name
        params = sorted({f"/{p}/{k}" for p, d in data.items() if isinstance(d, dict) for k in d})
        spec = {"t": "save", "name": name, "desc": recipe.get("desc", "composed from fragments: " + ", ".join(p["frag"] for p in recipe["parts"])), "params": params, "opts": opts, "components": comps, "data": data}
        os.makedirs(os.path.join(HOME, "cache"), exist_ok=True)
        tmp = os.path.join(HOME, "cache", f".compose-{name}.json")
        json.dump(spec, open(tmp, "w", encoding="utf-8"))
        r = subprocess.run([A2, "save", tmp], capture_output=True, text=True, timeout=60)
        os.remove(tmp)
        if '"ok":true' not in r.stdout.replace(" ", ""):
            sys.exit((r.stdout or r.stderr).strip()[:400])
        status = "saved"
        hashes[name] = sh
        os.makedirs(os.path.dirname(hfile), exist_ok=True)
        json.dump(hashes, open(hfile, "w", encoding="utf-8"))
    bump([p["frag"] for p in recipe["parts"]])
    out = {"name": name, "status": status}
    if show:
        r = subprocess.run([A2, "use", name, "--data", json.dumps(data), *extra], capture_output=True, text=True, timeout=60)
        out["use"] = (r.stdout or r.stderr).strip()[:200]
    print(json.dumps(out, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except KeyError as e:
        sys.exit("unresolved placeholder {{" + str(e.args[0]) + "}}: define it in the recipe's vars or part \"with\"")
