# plate

The one frosted plate every card sits on, plus size-specific layouts (`#stage[data-size=...]`).

## Changes and learned (read before every use)
- 11-16: chip-size stat rows (`#stage[data-size="chip"] .row:has(> .stat + .stat)`) used `grid-auto-columns: minmax(92px, 1fr)`, so the card was sized to 4 x 92 px whatever the text and long units spilled past the tiles. Now `minmax(max-content, 1fr)`; the 92 px floor moved to the tile as the `--chip-stat-min-w` token. Checked from tree rects: no overlap, all tiles inside the row.
- 11-16, user: "inconsistency in text sizes and text boldness", "min max text sizes aint obeyed". Every font size/weight here now uses the type scale (`var(--type-<role>)`, `var(--type-<role>-weight)`, ui/tokens/type.json; Tailwind text-<role>): same on screen in every card whatever its zoom, never under type-floor. Checked with `morc audit` (hidden, + --stress, + --type): 35/35 clean.
