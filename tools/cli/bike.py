# morc: show an interactive 3D bike (orbit, zoom, explode, spin, repaint). Usage: morc bike [--paint "#hex"]
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import BIN as A2  # noqa: E402

args = [A2, "use", "bike-viewer", "--max-frac", "0.8", "--ttl", "600"]
if "--paint" in sys.argv:
    args += ["--data", json.dumps({"paint": sys.argv[sys.argv.index("--paint") + 1]})]
sys.exit(subprocess.run(args).returncode)
