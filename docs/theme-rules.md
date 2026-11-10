# Theme rules: every component follows these, automatically

Read this before making or changing ANY component, chip or card. A new component that breaks one of these is not done.
The why behind each rule is in `design.md` (the dated notes); this page is the checklist. The glass theme is the approved default.

## Look
1. **One plate per card.** The card's root sits on `.card-plate`; pieces inside go flat. Never add your own background.
   3D cards are the exception: the model floats bare and only the containers beside it (title, controls, under the model) get plates.
   Any container can ask for its own plate with `"plate": true`.
2. **The skin decides the material.** Style a look under `html[data-skin~="<word>"]` (today: `glass`; `glass flat` = no texture).
   Never style a look with `html.os-mac` / `os-win`; that class is only for what truly needs the platform (dropping the CSS blur where
   native glass sits under the page).
3. **Colors are data.** Every color comes from `ui/themes.json` (base, then the skin, then `platforms.<os>`), read as a CSS variable.
   Accent-colored things use `--accent` (or a `color-mix` of it), so swapping the accent recolors everything. No color literals for a look.
4. **Sizes are data.** Every size that is a choice is a token in `ui/tokens/<component>.json`, read as `var(--name, fallback)`, and listed
   in the component's `metadata.json`.
5. **Glass pieces.** Rows are dark lens rows: a dark fill deepening to the edges (inset shadow), a black rim, a quiet grey inner line,
   white text, no accent line, no glow. Buttons and badges are FLAT accent fills with black text (`fill-ink`). Yes / No are a deep green
   and a deep coral-red (`yes-fill`, `no-fill`) with a darker rim of their own color. Corners are modest radii plus `corner-shape: squircle`;
   never pills.
6. **Palette.** Charts take colors from `charts/util.js` (`palette`, `base`, `accent`, `ink`, `themeNum`); everything else uses the same
   walk through `--pal-1..6` (theme.js). The glass skin walks 200 deg from the accent: pies and lines are colorful, bars keep the accent,
   sets of items (progress bars, paste numbers, done ticks, the line under each strip number) take one color each. Status colors come
   from `--good`, `--warn`, `--bad`. Colors go on items, never on the glass. Never pick a color in component code.
7. **The accent edge marks a compact card only.** Chips, working cards and charts keep a neutral edge. No accent glow ON the glass ever.

## Text and shapes
8. **One line, cut, full text in the tooltip.** Titles, labels and button text never wrap: `white-space: nowrap`, ellipsis,
   `min-width: 0` on the flex child, the full text as `title` by default.
9. **Even shapes.** Rows and buttons share a min height and the same padding (tokens). Truncate; do not widen.
10. **Card titles are centered** (`title-align`).
11. **No Close buttons.** Every card has the built-in dismiss (x); a card never adds its own Close.
12. **Numbers.** Several = ONE long strip (`stat-tiles`, size `chip`): a single row, the numbers straight on the glass. One number =
    `stat-chip`, a flat rectangle (`chip-radius`). Never a square of four.

## Size by purpose
13. Every chip says what it is in its seed `opts.size`: `compact` (a glance-and-go list), `medium` (a working card you read or use:
    pastes, to-dos, asks; a little bigger, `limits.medium-scale`), `large` (charts, graphs, 3D: big from the start, `limits.large-scale`),
    `chip` (stats, short status). No size: a plain list starts compact, everything else full.
14. **Key + detail (the mentality).** Compact shows a card's KEY part; hovering reveals the DETAIL, growing the way the card is long.
    Tall parts (lists, asks, steps) fold downward with a fading peek; wide parts fold to the right (`.fold-x` + `--key-w`: a pie's legend
    slides out beside the ring). Any card with a lot to show (natural height past `limits.compact-if-taller`) folds too. A new component
    only names its key part (a chart module exports `keyWidth(s)`, a list's key is its first rows) and gets the fold, the spring and the
    mirrored close from `core/compact.js`. Grown charts label every value.

## Motion
15. **Damped springs.** Grow and shrink run on a damped spring (`limits.spring-*`); opening and folding are mirrors (`compact.expand` /
    `compact.collapse`): the content moves with the spring and the window resizes only when nothing would jump.
16. **Arrival:** a card grows from nothing on a damped spring (`arrive-*`), overshoots a touch and rests, with one soft accent glow
    OUTSIDE the plate that only fades out. That is the only unasked motion; nothing loops for attention. Honor `prefers-reduced-motion`.
17. **Every hover rule is written `:is(:hover, .hov)`.** On macOS the page cannot see the pointer while our app is inactive; the host
    sends the pointer and the page marks what is under it `.hov`. A bare `:hover` silently does nothing there.
18. **Native glass follows the plate.** The page reports each `.card-plate` rect (`layout.glass`) and streams it during any animation
    (morph, arrival); never animate the plate's outer size any other way.
19. **Focusing one card never touches the others** (`crowd_yield` 1.0).

## Placement
20. **Fitting first.** `--at`, then the tier ladder (free room beside the ACTIVE window, sides before corners), then a remembered spot
    only if it is free, then the scorer (starts right, avoids every card up). Never drop overflow cards at one fixed spot. The active
    window is the one you work in, even while a card is frontmost.

## Done means
Layout JSON in `cache/seed/`, a short md in `docs/chips/`, a line in `docs/chips/INDEX.md`, tokens listed in `metadata.json`, checked on
screen (compact, hovered, a long label) with `morc` and a screenshot.
