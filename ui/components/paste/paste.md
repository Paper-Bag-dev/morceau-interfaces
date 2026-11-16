# paste

See metadata.json for what it is and its deps.

## Changes and learned (read before every use)
- 11-16, user: "inconsistency in text sizes and text boldness", "min max text sizes aint obeyed". Every font size/weight here now uses the type scale (`var(--type-<role>)`, `var(--type-<role>-weight)`, ui/tokens/type.json; Tailwind text-<role>): same on screen in every card whatever its zoom, never under type-floor. Checked with `morc audit` (hidden, + --stress, + --type): 35/35 clean.
- 11-16, user: "support bot is still so small". Cause: limits.json had min-zoom 0.7 (his 11-14 hard rule says 1) and resting folds down to 0.4, so it rested at 0.64. Now min-zoom 1, idle-min-zoom 0.85, and the type floor is judged at the zoom on screen (--zoom-now), resting fold included.
