# chart

See metadata.json for what it is and its deps.

## Changes and learned (read before every use)
- 11-16, user: "inconsistency in text sizes and text boldness", "min max text sizes aint obeyed". Every font size/weight here now uses the type scale (`var(--type-<role>)`, `var(--type-<role>-weight)`, ui/tokens/type.json; Tailwind text-<role>): same on screen in every card whatever its zoom, never under type-floor. Checked with `morc audit` (hidden, + --stress, + --type): 35/35 clean.
