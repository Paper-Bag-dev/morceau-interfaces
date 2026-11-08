# morc free: where is free space right now? (the host does the math; this just prints it small)
#
#   morc free           one line: mode, the active window, and the best free rectangle per region (tier 0: nothing sacrificed)
#   morc free --more    everything: work area, tier 1 (sacrificing non-active windows), all windows, toast zone
#
# Rects are [left, top, right, bottom] in physical px. Regions: right | below | left | above | gap (around the active window),
# or right | mid | left (screen thirds when fullscreen / no active window). Fill order: right, then down, then left.
# You rarely need this: just push with  morc use <name> --region right --align top-right --dx 0 --dy 0  (or --region auto)
# and the host places it. Use this only to decide what shape a widget should have.
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import BIN as A2  # noqa: E402

r = subprocess.run([A2, "space"], capture_output=True, text=True, timeout=30)
sp = json.loads(r.stdout or "{}")
if "--more" in sys.argv:
    print(json.dumps(sp, separators=(",", ":")))
else:
    best = {}
    cell = lambda n: not (n.endswith("-col") or n.endswith("-row"))   # the four bands duplicate the cells: only in --more
    for x in [x for x in sp.get("A", []) if cell(x["name"])]:
        best.setdefault(x["name"], x["rect"])          # tier 0 lists the largest first per name
    sacrificing = False
    if not best:                                       # a maximized window behind leaves nothing at tier 0: show what sacrificing non-active windows frees
        sacrificing = True
        for x in [x for x in sp.get("B", []) if cell(x["name"])]:
            best.setdefault(x["name"], x["rect"])
    print(json.dumps({"mode": sp.get("mode"), "active": sp.get("active"), "free": best, "sacrificing_other_windows": sacrificing}, separators=(",", ":")))
