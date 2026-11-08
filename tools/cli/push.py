# morc push: show several saved cards at once in the free space (the host places them). Usage: morc push NAME [NAME ...] | morc push --all [--prefix tok-]
"""Reads cards/NAME.json (written with `morc card`) and shows each with `morc use <chip> --data ... --region auto`. Size is the card's importance, never
below its min_importance (the floor you are not willing to go under unless the user says so). Many cards, one command."""
import glob
import json
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import APP, BIN, HOME  # noqa: E402

CARDS = os.path.join(HOME, "cards")
DEFAULT_MIN = 0.85  # text stays readable: no card goes below this size unless its own file says otherwise


def margin():
    """The host's margin from every edge (prefs/placement.json, in your data folder, else the shipped default 14), so flush alignment is data too."""
    for root in (HOME, APP):
        p = os.path.join(root, "prefs", "placement.json")
        if os.path.exists(p):
            try:
                return int(json.load(open(p, encoding="utf-8-sig")).get("margin", 14))
            except (ValueError, OSError):
                break
    return 14


MARGIN = margin()


def main(argv):
    prefix = argv[argv.index("--prefix") + 1] if "--prefix" in argv else ""
    names = [a for a in argv if not a.startswith("--") and a != prefix]
    if "--all" in argv:
        names = sorted(os.path.splitext(os.path.basename(p))[0] for p in glob.glob(os.path.join(CARDS, prefix + "*.json")))
    if not names:
        print(__doc__)
        return 2
    rc = 0
    for n in names:
        p = os.path.join(CARDS, n + ".json")
        if not os.path.exists(p):
            print("no card " + n + " (write it with: morc card " + n + " --chip <chip> --set ...)")
            rc = 1
            continue
        c = json.load(open(p, encoding="utf-8-sig"))
        imp = max(float(c.get("importance", 1.0)), float(c.get("min_importance", DEFAULT_MIN)))
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="ascii") as f:
            json.dump(c.get("data", {}), f)
        args = [BIN, "use", c["chip"], "--id", n, "--data", "@" + f.name, "--importance", str(imp), "--ttl", str(c.get("ttl", 90)), "--region", c.get("region", "auto")]
        if c.get("align"):
            args += ["--align", c["align"]]
        # Alignment is automatic: a card aligned left or right sits FLUSH with that edge of its quadrant. The host keeps a margin from every edge
        # (prefs/placement.json), so the card's own dx cancels it: -margin for left, +margin for right. An explicit dx in the card file wins.
        dx = c.get("dx")
        if dx is None and c.get("align") in ("left", "right"):
            dx = -MARGIN if c["align"] == "left" else MARGIN
        if dx is not None:
            args += ["--dx", str(dx)]
        if c.get("dy") is not None:
            args += ["--dy", str(c["dy"])]
        rc |= subprocess.run(args, stdout=subprocess.DEVNULL).returncode
        print(("shown " if rc == 0 else "failed ") + n)
    return rc


sys.exit(main(sys.argv[1:]))
