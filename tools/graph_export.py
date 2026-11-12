"""Turn the code-graph index into the generic morc graph schema (see docs/graphs.md). Prints JSON on stdout.

Reads <repo>/.code-graph/index.db READ-ONLY (written by code-graph-mcp; never modified here).

  py -I tools/graph_export.py --root morc/ --flow report                 files under morc/, the "report" route highlighted
  py -I tools/graph_export.py --root src/ --langs python --level dir   directories as nodes
  py -I tools/graph_export.py --symbol run --depth 2                     the call neighbourhood of every symbol named `run`

Schema (the same one any other source can produce, e.g. a goal map):
  {"title", "nodes":[{"id","label","group","size"}], "edges":[{"source","target","weight","kind"}],
   "flow":[node ids in order], "groups":[{"id","label"}], "stats":{...}}
Named flows live in tools/flows.json: {"report": ["ui/components/button.js", ...]} (paths relative to --root).
Re-index first if the code changed:  code-graph-mcp incremental-index   (run at the repo root).
"""
import argparse
import json
import os
import sqlite3
import sys
from collections import Counter, defaultdict, deque

HERE = os.path.abspath(os.path.dirname(__file__))
LANGS = {"rust", "javascript", "typescript", "python", "go", "java", "kotlin", "c", "cpp", "csharp", "ruby", "php"}


def find_db(start=HERE):
    d = start
    for _ in range(6):
        p = os.path.join(d, ".code-graph", "index.db")
        if os.path.exists(p):
            return p
        d = os.path.dirname(d)
    sys.exit("no .code-graph/index.db found: run `code-graph-mcp rebuild-index --confirm` at the repo root")


def skip(path):
    return "/vendor/" in path or ".min." in path or "/node_modules/" in path or "/target/" in path


def group_of(rel):
    d = os.path.dirname(rel).replace("\\", "/")
    return d or "(root)"


def file_graph(c, a):
    files = {r[0]: (r[1].replace("\\", "/"), r[2]) for r in c.execute("select id, path, language from files")}
    langs = set(a.langs.split(","))
    keep = {fid: p for fid, (p, lang) in files.items() if p.startswith(a.root) and lang in langs and not skip(p)}
    node_file, per_file = {}, Counter()
    for nid, fid in c.execute("select id, file_id from nodes"):
        if fid in keep:
            node_file[nid] = fid
            per_file[fid] += 1
    weight, kinds = Counter(), defaultdict(Counter)
    for s, t, rel in c.execute("select source_id, target_id, relation from edges"):
        fs, ft = node_file.get(s), node_file.get(t)
        if fs and ft and fs != ft:
            weight[(fs, ft)] += 1
            kinds[(fs, ft)][rel] += 1
    rel_of = lambda fid: keep[fid][len(a.root):]
    if a.level == "dir":
        dirs = sorted({group_of(rel_of(f)) for f in keep})
        size = Counter()
        for f in keep:
            size[group_of(rel_of(f))] += per_file[f]
        w2, k2 = Counter(), defaultdict(Counter)
        for (fs, ft), w in weight.items():
            ds, dt = group_of(rel_of(fs)), group_of(rel_of(ft))
            if ds != dt:
                w2[(ds, dt)] += w
                k2[(ds, dt)].update(kinds[(fs, ft)])
        nodes = [{"id": d, "label": d, "group": d.split("/")[0], "size": size[d]} for d in dirs]
        edges = [{"source": s, "target": t, "weight": w, "kind": k2[(s, t)].most_common(1)[0][0]} for (s, t), w in w2.items() if w >= a.min_weight]
    else:
        order = sorted(keep, key=lambda f: keep[f])
        nodes = [{"id": rel_of(f), "label": os.path.basename(keep[f]), "group": group_of(rel_of(f)), "size": per_file[f] or 1} for f in order]
        edges = [{"source": rel_of(fs), "target": rel_of(ft), "weight": w, "kind": kinds[(fs, ft)].most_common(1)[0][0]}
                 for (fs, ft), w in sorted(weight.items(), key=lambda kv: -kv[1]) if w >= a.min_weight]
    return nodes, edges


def symbol_graph(c, a):
    files = {r[0]: r[1].replace("\\", "/") for r in c.execute("select id, path from files")}
    info = {}
    for nid, fid, name, qn, typ in c.execute("select id, file_id, name, qualified_name, type from nodes"):
        info[nid] = (fid, name, qn or name, typ)
    seeds = [n for n, (fid, name, qn, typ) in info.items() if name == a.symbol and (not a.root or files.get(fid, "").startswith(a.root))]
    if not seeds:
        sys.exit(f"no symbol named {a.symbol!r} under {a.root!r}")
    adj = defaultdict(set)
    rows = [(s, t, rel) for s, t, rel in c.execute("select source_id, target_id, relation from edges") if rel in ("calls", "references")]
    for s, t, _ in rows:
        adj[s].add(t)
        adj[t].add(s)
    seen = {n: 0 for n in seeds}
    q = deque(seeds)
    while q:
        n = q.popleft()
        if seen[n] >= a.depth:
            continue
        for m in adj[n]:
            if m not in seen and not skip(files.get(info[m][0], "")):
                seen[m] = seen[n] + 1
                q.append(m)
    nid_of = lambda n: f"{files.get(info[n][0], '?')}::{info[n][2]}"
    nodes = [{"id": nid_of(n), "label": info[n][1], "group": group_of(files.get(info[n][0], "?")), "size": 3 if n in seeds else 1} for n in seen]
    es = Counter()
    for s, t, rel in rows:
        if s in seen and t in seen:
            es[(nid_of(s), nid_of(t), rel)] += 1
    edges = [{"source": s, "target": t, "weight": w, "kind": rel} for (s, t, rel), w in es.items()]
    return nodes, edges


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default="morc/", help="only files under this path prefix (repo-relative)")
    ap.add_argument("--langs", default="rust,javascript")
    ap.add_argument("--level", choices=["file", "dir"], default="file")
    ap.add_argument("--flow", help="named route from tools/flows.json to highlight")
    ap.add_argument("--symbol", help="graph the call neighbourhood of this symbol instead of files")
    ap.add_argument("--depth", type=int, default=2)
    ap.add_argument("--min-weight", type=int, default=1)
    ap.add_argument("--title")
    ap.add_argument("--repo", help="graph another repo: read its .code-graph/index.db (index it there first)")
    a = ap.parse_args()
    if a.root and not a.root.endswith("/"):
        a.root += "/"

    uri = "file:" + find_db(os.path.abspath(os.path.expanduser(a.repo)) if a.repo else HERE).replace("\\", "/")
    try:
        c = sqlite3.connect(uri + "?mode=ro", uri=True)
        c.execute("select 1 from files limit 1")
    except sqlite3.OperationalError:
        # a WAL-mode index whose -shm/-wal side files do not exist cannot be opened read-only (it may not create them):
        # read the file as it is instead. Still never writes.
        c = sqlite3.connect(uri + "?mode=ro&immutable=1", uri=True)
    nodes, edges = symbol_graph(c, a) if a.symbol else file_graph(c, a)

    flow = []
    if a.flow and not a.symbol:
        cfg = json.load(open(os.path.join(HERE, "flows.json"), encoding="utf-8"))
        ids = {n["id"] for n in nodes}
        flow = [p for p in cfg.get(a.flow, []) if p in ids]
    groups = [{"id": g, "label": g} for g in sorted({n["group"] for n in nodes})]
    title = a.title or (f"Calls around {a.symbol}" if a.symbol else f"{a.root.rstrip('/')} ({a.level}s)")
    print(json.dumps({"title": title, "nodes": nodes, "edges": edges, "flow": flow, "groups": groups,
                      "stats": {"nodes": len(nodes), "edges": len(edges), "kinds": dict(Counter(e["kind"] for e in edges))}}))


if __name__ == "__main__":
    main()
