# paste-blocks

Paste blocks to hand over in order: prompts, tool Input/Output/reply text, request bodies. One numbered row per block with its size and a Copy
button; a click on a row opens the exact text (scrollable, selectable). A copied row stays marked, so the user sees what is done. Layout:
`cache/seed/paste-blocks.json`, component `ui/components/paste/`.

**Data:** `{"title": "Support bot: prompt v3", "body": "Studio, Build tab", "items": [{"title": "General instruction", "text": "...", "note": "replaces all"}]}`
**Push:** write the data to a JSON file (long, multi-line text is safe there; never inline) and run `morc use paste-blocks --data @file.json`.
**Copy:** the host copies with `pbcopy` on macOS (WKWebView may refuse the browser clipboard on a morc:// page); Windows uses the browser clipboard.
**Rule:** one card per handover, blocks in the order to paste them; close it (`morc close paste-blocks`) when the user is done.
**Size (user, 11-11):** `medium` + `opts.idle` 0.5 in the seed: it rests at half size while unused and springs to full size on hover (the "squeezable" ask). After editing the seed run `morc save cache/seed/paste-blocks.json`: the host reads `cache/components/`, not the seed.
**Scroll (fixed 11-11):** paste lists never fold (`ui/components/plate/plate.css`): in a compact card they keep scrolling, so one long list is fine again. Squeeze the card with `opts.idle`, not the fold.
**Timing (11-11):** paste cards are `medium`, so they use the per-kind limits in `ui/limits.json` (`medium.*`): open after 60 ms, hold 3 s after the pointer leaves, a drag needs a 350 ms press (a click never turns into a drag).
**Following (11-11):** each Copy also emits `{kind: "paste_copied", row, of, title}` to the inbox (no wake), so the session can see how far the handover got (`grep paste_copied inbox.jsonl | tail`).
