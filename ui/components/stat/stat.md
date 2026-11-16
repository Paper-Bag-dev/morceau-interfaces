# stat

One hero numeral with a label, unit and optional needle; several stats in one row become a hero grid (tokens in `ui/tokens/stat.json`).

## Changes and learned (read before every use)
- 11-16, user: "text kinda leaks outta the black rounded bg … always use like a flex so u never have to hard code stuff and it dynamically sizes". Cause: typed-in `min-width: 170px`, grid columns fixed by a 118 px floor shared equally, and label/unit cut with `nowrap` + ellipsis. Fix: `min-width` is the `--stat-min-w` token, grid columns are `minmax(max-content, 1fr)` (a column is never narrower than its text), the tile floor is the `--stat-tile-min-w` token, and labels/units are never cut or ellipsized. Chip-size cards had their own override in `plate/plate.css` (see there).
- Verify from the tree's numbers: tiles must not overlap and must sit inside their row (`morc tree <id>` rects). The tree's flags did NOT report the old hidden overflow.
- 11-16, user: "inconsistency in text sizes and text boldness", "min max text sizes aint obeyed". Every font size/weight here now uses the type scale (`var(--type-<role>)`, `var(--type-<role>-weight)`, ui/tokens/type.json; Tailwind text-<role>): same on screen in every card whatever its zoom, never under type-floor. Checked with `morc audit` (hidden, + --stress, + --type): 35/35 clean.
- 11-16: the type scale now cancels only the BASE zoom (`window.__baseZoom` = host scale x size kind, set where the base zoom is decided); the resting fold, hover grow and fit zoom text with the card again (user asked "did the animations of growing and shrinking die down?": they had, for text). Proven from tree zoom: donut-card 0.889 -> 1.111 on hover -> 0.889 after the shrink delay.
