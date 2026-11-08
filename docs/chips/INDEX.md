# UI chips: read this first

A **chip** is a ready-made card: a JSON file (the layout, already built) plus a short md next to this index. You do not write component code.

**Group by kind (the user has said this many times):** numbers and text go together in ONE card (`stat-tiles`); pictures (charts) each go in their own
separate card (`donut-card`, `week-chart`). Never mix a picture and a text list in one card, never stack everything as one long vertical story, never
one big blocker. Several small cards spread into the free space (`--region auto`), sized in between (about importance 1.0; text cards 0.85 to 1.0).

**To show something:**
1. Find the chip below. Open its md (one screen) for the params and the size knobs.
2. Change only its **data** (a small JSON file with the params) and, if needed, a size knob. The layout JSON stays untouched.
3. Push it: `morc use <chip> --data @file.json` (add `--importance 0.7` to make it smaller, `--importance 1.2` bigger).
4. If no chip fits, make one: write its layout JSON in `cache/seed/`, a short md here, and one line below. Do not write a one-off spec.

| chip | use it for | params (data) | md |
|---|---|---|---|
| `tokens-dashboard` | Claude token usage: `morc tokens` pushes SEVERAL SMALL cards into free space (default); the one big card only with `--one-card` | total, today, reuse, replies, models, days | [tokens-dashboard.md](tokens-dashboard.md) |
| `donut-card` | a share-of-a-whole donut | title, values, labels | [donut-card.md](donut-card.md) |
| `stat-card-min` | one big number with a label and unit | h.label, h.value, h.unit | (data only: `{"h": {"label": "New today", "value": 4.4, "unit": "M"}}`) |
| `week-chart` | any 7-point bar or line chart | title, values, labels, unit | [week-chart.md](week-chart.md) |
| `ask-permission` | an urgent yes/no for the user | title, body, yes, no, ctx | [ask-permission.md](ask-permission.md) |
| `graph-view` | a node-link graph (code, goals, flows) | title, graph | [graph-view.md](graph-view.md) |
| `bike-viewer` | the interactive 3D bike demo | title, paint, spin, explode | [bike-viewer.md](bike-viewer.md) |
| `skeleton-card` | an empty shimmering placeholder popup, shown under the id the real card will use | none | [skeleton-card.md](skeleton-card.md) |

**Rules that apply to every chip** (details in `../design.md` and `../learnings.md`): never clip, size within the container, text at least 15 px, explain each
metric in a line, one card per kind of thing (do not stack unlike things in one card), short `ttl`, never cover your own cards.
