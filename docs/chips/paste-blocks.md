# paste-blocks

Paste blocks to hand over in order: prompts, tool Input/Output/reply text, request bodies. One numbered row per block with its size and a Copy
button; a click on a row opens the exact text (scrollable, selectable). A copied row stays marked, so the user sees what is done. Layout:
`cache/seed/paste-blocks.json`, component `ui/components/paste/`.

**Data:** `{"title": "Support bot: prompt v3", "body": "Studio, Build tab", "items": [{"title": "General instruction", "text": "...", "note": "replaces all"}]}`
**Push:** write the data to a JSON file (long, multi-line text is safe there; never inline) and run `morc use paste-blocks --data @file.json`.
**Copy:** the host copies with `pbcopy` on macOS (WKWebView may refuse the browser clipboard on a morc:// page); Windows uses the browser clipboard.
**Rule:** one card per handover, blocks in the order to paste them; close it (`morc close paste-blocks`) when the user is done.
