# morc card: write or update ONE card's data file (cards/<name>.json) in a single quick command; it is remembered until you change it. Usage: morc card NAME [--chip C] [--set a.b=VALUE ...] [--data @file.json] [--importance X] [--min-importance X] [--region R] [--ttl N]
"""A card is a small JSON file in your data folder (cards/NAME.json): which chip it uses, its data, and its size floor. Write it once, then only
change what changed with --set. `morc push NAME...` shows one or many cards in the free space. Nothing here ever rewrites a chip's layout."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _home import HOME  # noqa: E402

CARDS = os.path.join(HOME, "cards")


def value(text):
    try:
        return json.loads(text)
    except ValueError:
        pass
    # Windows shells strip the quotes inside JSON, so a plain list is fine too: [a,b,c] or [1,2,3]
    if text.startswith("[") and text.endswith("]"):
        items = []
        for t in (s.strip().strip("\"'") for s in text[1:-1].split(",") if s.strip()):
            try:
                items.append(json.loads(t))
            except ValueError:
                items.append(t)
        return items
    return text


def set_path(d, path, v):
    keys = path.strip("/").replace("/", ".").split(".")
    for k in keys[:-1]:
        d = d.setdefault(k, {})
    d[keys[-1]] = v


def main(argv):
    if not argv or argv[0].startswith("--"):
        print(__doc__)
        return 2
    name, rest = argv[0], argv[1:]
    os.makedirs(CARDS, exist_ok=True)
    path = os.path.join(CARDS, name + ".json")
    card = {}
    if os.path.exists(path):
        card = json.load(open(path, encoding="utf-8-sig"))
    card.setdefault("data", {})
    i = 0
    while i < len(rest):
        a, nxt = rest[i], (rest[i + 1] if i + 1 < len(rest) else "")
        if a == "--chip":
            card["chip"] = nxt
        elif a == "--set":
            k, _, v = nxt.partition("=")
            set_path(card["data"], k, value(v))
        elif a == "--data":
            src = json.load(open(nxt[1:] if nxt.startswith("@") else nxt, encoding="utf-8-sig"))
            card["data"].update(src)
        elif a in ("--importance", "--min-importance", "--ttl"):
            card[a[2:].replace("-", "_")] = float(nxt) if a != "--ttl" else int(nxt)
        elif a in ("--dx", "--dy"):  # nudge from the aligned spot in physical px (e.g. flush with the quadrant edge)
            card[a[2:]] = int(nxt)
        elif a == "--region":
            card["region"] = nxt
        elif a == "--align":  # left | right: where the card's contents start (titles, charts, Close line up on that edge); auto (the default) = the edge nearest the active window
            if nxt == "auto":
                card.pop("align", None)
            else:
                card["align"] = nxt
        else:
            print("unknown option " + a)
            return 2
        i += 2
    if "chip" not in card:
        print("a new card needs --chip <name> (see docs/chips/INDEX.md)")
        return 2
    json.dump(card, open(path, "w", encoding="utf-8"), indent=1)
    print(path)
    return 0


sys.exit(main(sys.argv[1:]))
