# morc sketchfab: find and download 3D models from Sketchfab. It only fetches assets; showing one is a RECIPE (see cache/recipes/pikachu-sketchfab.json):
# a recipe loads the model's attribution.json as variables ("vars") and puts it in the generic `gltf-model` fragment, so a new model or look is
# data and flags, never a code change:   morc compose --recipe <recipe> --use -- --region right   [--set m.fps=8] [--set m.anim=Dance]
#
#   morc sketchfab search <query> [--faces N] [--n N]    compact list: uid | name | author | licence | faces | animations
#   morc sketchfab get <uid>                              download the glTF into ui/models/assets/<uid>/ and write attribution.json
#
# `get` needs a Sketchfab API token: SKETCHFAB_TOKEN from the environment or from morc/.env (git-ignored, see .env.example). Never printed.
# Prefer light models (under ~20k faces): the page renders in software GL. Models are CC-licensed; keep the credit line.
import io
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import APP, HOME  # noqa: E402

ROOT = APP
ASSETS = os.path.join(ROOT, "ui", "models", "assets")
UA = {"User-Agent": "morc"}


def _load_env():
    """Read morc/.env (git-ignored) into the environment; a variable already set in the environment wins."""
    p = next((q for q in (os.path.join(HOME, ".env"), os.path.join(APP, ".env")) if os.path.exists(q)), os.path.join(HOME, ".env"))
    if os.path.exists(p):
        for line in open(p, encoding="utf-8-sig"):
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, v = line.split("=", 1)
                os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


_load_env()


def api(path, auth=False):
    h = dict(UA)
    if auth:
        tok = os.environ.get("SKETCHFAB_TOKEN", "")
        if not tok:
            sys.exit("no SKETCHFAB_TOKEN: put it in morc/.env (see .env.example) or set the environment variable")
        h["Authorization"] = "Token " + tok
    try:
        return json.load(urllib.request.urlopen(urllib.request.Request("https://api.sketchfab.com" + path, headers=h), timeout=60))
    except urllib.error.HTTPError as e:
        sys.exit(f"sketchfab: HTTP {e.code} on {path.split('?')[0]}")


def opt(name, default):
    a = sys.argv
    return type(default)(a[a.index(name) + 1]) if name in a else default


def search(q):
    p = urllib.parse.urlencode({"type": "models", "q": q, "downloadable": "true", "sort_by": "-likeCount", "count": opt("--n", 12), "max_face_count": opt("--faces", 30000), "archives_max_face_count": opt("--faces", 30000)})
    for m in api("/v3/search?" + p)["results"]:
        print(m["uid"], "|", m["name"][:36], "|", m["user"]["displayName"][:18], "|", (m.get("license") or {}).get("label"), "| faces", m.get("faceCount"), "| anim", m.get("animationCount"))


def get(uid):
    folder = os.path.join(ASSETS, uid)
    meta_file = os.path.join(folder, "attribution.json")
    if os.path.exists(meta_file):
        return json.load(open(meta_file, encoding="utf-8"))
    info = api(f"/v3/models/{uid}")
    dl = api(f"/v3/models/{uid}/download", auth=True)
    url = (dl.get("gltf") or {}).get("url")
    if not url:
        sys.exit("this model has no glTF download")
    data = urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=180).read()
    os.makedirs(folder, exist_ok=True)
    zipfile.ZipFile(io.BytesIO(data)).extractall(folder)           # extractall strips absolute paths and '..'
    gltf = next((os.path.relpath(os.path.join(r, f), folder).replace("\\", "/") for r, _, fs in os.walk(folder) for f in fs if f.endswith(".gltf")), None)
    if not gltf:
        sys.exit("downloaded, but no .gltf inside")
    meta = {"uid": uid, "name": info.get("name"), "author": (info.get("user") or {}).get("displayName"), "licence": (info.get("license") or {}).get("label"),
            "faces": info.get("faceCount"), "url": info.get("viewerUrl"), "gltf": gltf, "src": f"models/assets/{uid}/{gltf}"}
    json.dump(meta, open(meta_file, "w", encoding="utf-8"), indent=2)
    return meta


def main():
    a = sys.argv[1:]
    if len(a) >= 2 and a[0] == "search":
        search(a[1])
    elif len(a) >= 2 and a[0] == "get":
        m = get(a[1])
        print(json.dumps({k: m[k] for k in ("uid", "name", "author", "licence", "faces", "src")}, separators=(",", ":")))
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
