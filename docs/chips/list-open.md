# list-open

The reading card: a pill list that shows every row in full (text wraps, no ellipsis, no fold, zoom never under 1). Use it for info-dense
facts the user asked to read (an analysis, a verdict, a breakdown). Layout: `cache/components/list-open.json` (list `wrap: true`, size `large`).

**Size (hard rule, docs/theme-rules.md 12c):** size A, which the user picked from three on screen on 11-13: row title 13 px, detail line
11.5 px, list 300 to 320 px, card about 410 css px. Min 12.5 / 11.5 px, max 17 / 14 px (only on request). Check `zoom` = 1 in `morc tree`.

**Data:** `{"title": "...", "items": [{"text": "headline", "sub": "detail line", "badge": "tag"}], "vars": {}}` (max 8 rows; `vars`
overrides size tokens for this card only, e.g. `{"row-title-size-wrap": "15px"}`).
**Push:** `morc use list-open --id <id> --force --pin --data @file.json --region left|mid|right` (asked-for cards are forced).
**Several:** one per third, left to right in reading order; verify no overlap in `morc status`.
