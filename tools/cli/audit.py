# morc audit: show every chip (or the named ones) with its sample data, read its tree and report what a screenshot would show wrong. Usage: morc audit [CHIP ...] [--stress] [--json] [--type]
"""For each chip in cache/seed: `morc use` it hidden (`--hidden`: laid out and placed, window never shown), read `morc tree <id>`, then close it. Reports per chip: flags (cut, spills,
clipped, offcard, empty, not-rendered), siblings that overlap, and children sticking out of their parent. `--stress` doubles every string in
the sample data so long labels and units are tested too. Nothing is written except the report; verification comes from the tree, never a
screenshot (docs/trees.md)."""
import json
import os
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import APP, BIN  # noqa: E402

BAD = {"cut", "spills", "tiny", "clipped", "offcard", "empty", "not-rendered"}
SKIP_STRESS = ("http", "www.", "/", "\\", "#", "{")  # urls, paths, colours, binds: doubling them breaks the card (11-16: a doubled video url started playing)


def morc(*args):
    r = subprocess.run([BIN, *args], capture_output=True, text=True, encoding="utf-8", errors="replace")
    try:
        return json.loads(r.stdout)
    except ValueError:
        return {"ok": False, "err": (r.stdout or r.stderr).strip()[:200]}


def stress(v):
    if isinstance(v, str):
        return v if v.startswith(SKIP_STRESS) or "://" in v or not v.strip() else f"{v} {v}"
    if isinstance(v, list):
        return [stress(x) for x in v]
    if isinstance(v, dict):
        return {k: stress(x) for k, x in v.items()}
    return v


def nodes(n, parent=None, out=None):
    out = [] if out is None else out
    if isinstance(n, dict):
        if "rect" in n and "type" in n:
            out.append((n, parent))
            parent = n
        for v in n.values():
            nodes(v, parent, out)
    elif isinstance(n, list):
        for v in n:
            nodes(v, parent, out)
    return out


def check(tree):
    found = []
    ns = nodes(tree)
    for n, p in ns:
        for f in set(n.get("flags") or []) & BAD:
            found.append(f"{n['id']}: {f}" + (f" ({n['issues'][0]['text']})" if n.get("issues") else ""))
        if p:
            x, y, w, h = n["rect"]; px, py, pw, ph = p["rect"]
            if w and h and (x < px - 1 or y < py - 1 or x + w > px + pw + 1 or y + h > py + ph + 1):
                found.append(f"{n['id']}: outside parent {p['id']}")
    kids = {}
    for n, p in ns:
        if p:
            kids.setdefault(p["id"], []).append(n)
    for ks in kids.values():
        for i, a in enumerate(ks):
            for b in ks[i + 1:]:
                ax, ay, aw, ah = a["rect"]; bx, by, bw, bh = b["rect"]
                if aw and ah and bw and bh and ax < bx + bw - 1 and bx < ax + aw - 1 and ay < by + bh - 1 and by < ay + ah - 1:
                    found.append(f"{a['id']} overlaps {b['id']}")
    return found


def fonts_by_type(tree, chip, acc):
    """--type: text sizes/weights per component type across chips, so drift between old and new components shows as data."""
    for n, _ in nodes(tree):
        for f in n.get("fonts") or []:
            acc.setdefault(n["type"], {}).setdefault(f, set()).add(chip)


def main():
    args = sys.argv[1:]
    do_stress, as_json, by_type = "--stress" in args, "--json" in args, "--type" in args
    type_acc = {}
    names = [a for a in args if not a.startswith("--")]
    seed_dir = os.path.join(APP, "cache", "seed")
    if not names:
        names = sorted(f[:-5] for f in os.listdir(seed_dir) if f.endswith(".json"))
    report = {}
    for name in names:
        spec = json.load(open(os.path.join(seed_dir, name + ".json"), encoding="utf-8-sig"))
        data = dict(spec.get("data") or {})
        for k in ("src", "url", "video"):  # never start media while auditing: the video component autoplays any source it gets
            if isinstance(data.get(k), str):
                data[k] = ""
        if do_stress:
            data = stress(data)
        cid = "audit-" + name
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
            json.dump(data, f)
            tmp = f.name
        shown = morc("use", name, "--id", cid, "--data", "@" + tmp, "--force", "--hidden", "--ttl", "20")  # never shown: laid out and measured only
        os.unlink(tmp)
        if not shown.get("ok"):
            report[name] = [f"not shown: {shown.get('err') or shown}"]
            continue
        time.sleep(1.2)
        tree = morc("tree", cid)
        report[name] = check(tree) if tree.get("ok") else [f"no tree: {tree.get('err')}"]
        if by_type and tree.get("ok"):
            fonts_by_type(tree, name, type_acc)
        morc("close", cid)
    if by_type:
        types = {t: {f: sorted(c) for f, c in fs.items()} for t, fs in type_acc.items()}
        if as_json:
            print(json.dumps(types, indent=1))
            return
        for t, fs in sorted(types.items()):
            print(f"{t}: " + "  ".join(f"{f} ({len(c)})" for f, c in sorted(fs.items(), key=lambda x: -len(x[1]))))
        return
    if as_json:
        print(json.dumps(report, indent=1))
        return
    bad = {k: v for k, v in report.items() if v}
    for k, v in report.items():
        print(f"{'FAIL' if v else 'ok  '} {k}" + ("".join(f"\n       - {x}" for x in v[:6]) if v else ""))
    print(f"\n{len(report) - len(bad)}/{len(report)} chips clean" + (" (stress)" if do_stress else ""))


if __name__ == "__main__":
    main()
