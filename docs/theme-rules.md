# Theme rules: every component follows these, automatically

Read this before making or changing ANY component, chip or card. A new component that breaks one of these is not done.
The why behind each rule is in `design.md` (the dated notes); this page is the checklist.

## Look
1. **One plate per card.** The card's root sits on `.card-plate`; pieces inside go flat. Never add your own background.
2. **The skin decides the material.** Style the look under `html[data-skin="<name>"]` (today: `glass`). Never style a look with
   `html.os-mac` / `os-win`; that class is only for what truly needs the platform (dropping the CSS blur where native glass sits under the page).
3. **Colors are data.** Every color comes from `ui/themes.json` (base, the skin's base, `platforms.<os>`), read as a CSS variable.
   Accent-colored things use `--accent` (or a `color-mix` of it), so swapping the accent recolors everything. No hex/rgb literals for a look.
4. **Sizes are data.** Every size that is a choice is a token in `ui/tokens/<component>.json`, read as `var(--name, fallback)`. List the
   tokens in the component's `metadata.json`.
5. **Glass skin pieces:** rows and plain pieces are dark lens rows (dark fill deepening to the edges via an inset shadow, black rim, a quiet
   grey inner line, white text, no accent line, no glow). Buttons and badges are FLAT accent fills with black text (`fill-ink`). Corners are
   modest radii (`row-radius`, `btn-radius`, `badge-radius`) plus `corner-shape: squircle`.
6. **Charts and graphs** take colors from `charts/util.js` (`palette`, `base`, `accent`, `ink`, `themeNum`): one family from the accent,
   spread set by the theme (`chart-hue-spread`, `chart-light-spread`). Status colors come from `--good`, `--warn`, `--bad`. Never pick a
   color in chart code.

## Text and sizing
7. **One line, cut, full text in the tooltip.** Titles, labels and button text never wrap into uneven shapes: `white-space: nowrap`,
   ellipsis, `min-width: 0` on the flex child, and the full text as `title` by default.
8. **Even shapes.** Rows, pills and buttons share a min height and the same padding (tokens). Cards come out a little taller than wide
   when grown and wider than tall when compact; truncate, do not widen.
9. **Card titles are centered** (`title-align`).

## Size by purpose
14. Every chip says what it is in its seed `opts.size`: `compact` (a glance-and-go list: starts small, opens on hover), `medium` (a working
    card you read or use: pastes, to-dos, asks: never squeezed), `large` (charts and graphs: big from the start, grow a little on hover).
    No size: a plain list starts compact, everything else at full size. Focusing one card never shrinks the others.
15. Several stats in one row are a hero grid (centered tiles, big numeral, small unit, one-line caption; tokens/stat.json). A card that
    ends in a lone button centers it.
16. **No Close buttons.** Every card has the built-in dismiss (x); a card never adds its own Close. **The accent edge marks a compact list card
    only**; chips (`opts.size: "chip"`: stats, short status: one rectangular row, white hero numerals, grey captions, no tiles), working
    cards (`medium`, a little bigger: `limits.medium-scale`) and charts keep a neutral edge.
17. **Numbers:** several = ONE long strip (`stat-tiles`, size chip): a single row, the numbers straight on the glass (no box per number).
    One number = `stat-chip`, a flat rectangle (`chip-radius`, never a pill). Never a square of four. Charts and graphs (`large`) start big (`limits.large-scale`), grow on
    hover and then show every value.
18. Skins are words: `glass`, or `glass flat` for the same glass without texture. Chart text uses the skin's `chart-ink` (light on glass).

## Behavior
10. **Every hover rule is written `:is(:hover, .hov)`.** On macOS the page cannot see the pointer while our app is inactive; the host
    sends the pointer and the page marks what is under it `.hov`. A bare `:hover` silently does nothing there.
11. **Compact first.** Cards show compact (`limits.compact`) and open on hover. Anything list-like joins the compact rules in
    `components/plate/plate.css` (truncate to a few rows, a fading peek) and the selector list in `core/compact.js`. Opening and folding
    are mirrors (`compact.expand` / `compact.collapse`): nothing pops, nothing jumps.
12. **Motion is a damped spring** (`limits.spring-*`) or a short ease, triggered by the user (hover, press, arrival). Never loop for
    attention; the only unasked motion is the arrival: a spring + fade, and one soft accent glow OUTSIDE the
    plate that only fades out (`arrive-*`). Never put an accent glow ON the glass (it colors the plate). Honor `prefers-reduced-motion`.
13. **Native glass follows the plate.** The page reports each `.card-plate` rect (`layout.glass`) and streams it during a morph; do not
    animate the plate's outer size any other way.

## Done means
Layout JSON in `cache/seed/`, a short md in `docs/chips/`, a line in `docs/chips/INDEX.md`, tokens listed in `metadata.json`, checked on
screen (compact, hovered, a long label) with `morc` and a screenshot.
