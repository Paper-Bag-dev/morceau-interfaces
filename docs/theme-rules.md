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
   white text, no accent line, no glow. Buttons and badges are FLAT accent fills with black text (`fill-ink`). Yes / No follow the ACCENT (user, 11-11): Yes is the flat accent fill, No a dark lens button with light text
   and a faint accent rim; never a separate green / red or yellow theme.
   A card may colour any button by name with `"tone": "green" | "yellow" | "blue" | "purple" | "red"` (themes.json base `tone-*`): a flat
   fill in that colour with dark ink. With no tone, the filled buttons of a card take the theme's `tone-cycle` in order
   (accent, green, blue, purple, yellow; `core/tones.js`), so several actions use the accent colours, not one colour repeated.
   Filled buttons are FLAT and follow the theme: one dark INNER rim of their own colour (`btn-rim-*`), no drop shadow, no highlight. Corners are modest radii plus `corner-shape: squircle`;
   never pills.
6. **Palette.** Charts take colors from `charts/util.js` (`palette`, `base`, `accent`, `ink`, `themeNum`); everything else uses the same
   walk through `--pal-1..6` (theme.js). The glass skin walks 200 deg from the accent: pies and lines are colorful, bars keep the accent,
   sets of items (progress bars, paste numbers, done ticks, the line under each strip number) take one color each. Status colors come
   from `--good`, `--warn`, `--bad`. Colors go on items, never on the glass. Never pick a color in component code.
7. **The accent edge marks a folded card only, as a shimmer (user, 11-11).** A folded card (rule 14) of ANY size gets a faint accent rim
   (`edge-base-a`) with a light streak that travels slowly round it and fades out at its tail (`edge-shimmer-*`, plate.css). It is the
   one looping motion allowed: it says "hover me, there is more". Open cards, chips and cards with nothing folded keep a neutral edge.
   No accent glow ON the glass ever.

## Text and shapes
8. **One line, cut, full text in the tooltip.** Titles, labels and button text never wrap: `white-space: nowrap`, ellipsis,
   `min-width: 0` on the flex child, the full text as `title` by default.
9. **Even shapes.** Rows and buttons share a min height and the same padding (tokens). Truncate; do not widen.
10. **Card titles are centered** (`title-align`).
11. **No Close buttons.** Every card has the built-in dismiss (x); a card never adds its own Close.
12. **Numbers.** Several = ONE long strip (`stat-tiles`, size `chip`): a single row, the numbers straight on the glass. One number =
    `stat-chip`, a flat rectangle (`chip-radius`). Never a square of four.
12b. **Text budget, max and min (user, 11-12). Check it before EVERY push.**
    - Max: card title at most 40 characters, each row or label at most 60, the whole card at most 120 words, with at most one muted
      caption line. Over budget: fold it into a pill list (`list-long`), trim it or split it into another card; never shrink the type.
    - Min: every card has a title, and every number has a unit and a label (`387 GB`, "Disk use"). A caption is not required.
    - Type sizes stay at the floors (card text 15 px, charts 13 px, hero 32 to 36 px); no maximum.

## Size by purpose
13. Every chip says what it is in its seed `opts.size` (user, 11-11):
    - `chip`: stats, short status (`limits.base-scale`).
    - `medium`: anything you READ or use: text-heavy cards, pill lists, pastes, to-dos, asks, and pies (`limits.medium-scale`).
    - `large`: line, bar and stacked charts, dashboards, graphs, 3D (`limits.large-scale`).
    No size named: a card with more text than `limits.text-heavy-chars` (and no chart) becomes `medium` by itself (`bridge.js autoSize`);
    anything else uses `base-scale`. Text-heavy content is never paragraphs: write it as a pill list (`list-long`: headline, muted detail
    line, badge), no tick boxes unless something is to be done.
14. **Key + detail, no hover scaling (user, 11-11). Apply to EVERY card you generate, any size.** A card with more to show starts folded
    on its KEY part and hovering reveals the DETAIL; leaving folds it back (`core/compact.js`, `bridge.js` FOLDS). The key parts:
    - pie / donut: the ring and its total; the legend slides out to the right.
    - line / area / bar chart: the same chart drawn narrower (`limits.chart-key-frac`); hover widens it and labels every value.
    - list / pill list / checklist / pastes / asks: the first rows with the next one fading; the rest slides down on hover.
    - any card taller than `limits.compact-if-taller`: its top part.
    The zoom NEVER changes on hover: only content opens and the window follows it. A new chart kind names its key part with
    `keyWidth(s)`; a new list-like component adds its class to the fold rules in plate.css. Folded cards wear the shimmer edge (rule 7).

## Motion
15. **Opening and folding are mirrors.** Content slides open and shut over `limits.reveal-ms` (CSS transitions in plate.css); the
    window follows the content and its shape stays the full window while anything moves.
16. **Arrival:** a card grows in its own shape on a damped spring (`arrive-*`): wide cards unfold left to right, long cards unroll top
    to bottom, others grow from the centre; list, checklist, paste and ask rows then come in one by one (`row-stagger-ms`). While it
    runs the window shape stays the full window (else the growth is clipped away). It overshoots a touch and rests, with one soft accent glow
    OUTSIDE the plate that only fades out. Besides the folded-edge shimmer (rule 7) nothing loops for attention. Honor `prefers-reduced-motion`.
17. **Every hover rule is written `:is(:hover, .hov)`.** On macOS the page cannot see the pointer while our app is inactive; the host
    sends the pointer and the page marks what is under it `.hov`. A bare `:hover` silently does nothing there.
18. **Native glass follows the plate.** The page reports each `.card-plate` rect (`layout.glass`) and streams it during any animation
    (morph, arrival); never animate the plate's outer size any other way.
19. **Focusing one card never touches the others.** The crowd shrink (others yield while one is hovered) was removed (user, 11-11).

## Placement
20. **Fitting first.** `--at`, then the tier ladder (free room beside the ACTIVE window, sides before corners), then a remembered spot
    only if it is free, then the scorer (starts right, avoids every card up). Never drop overflow cards at one fixed spot. The active
    window is the one you work in, even while a card is frontmost.

## Done means
Layout JSON in `cache/seed/`, a short md in `docs/chips/`, a line in `docs/chips/INDEX.md`, tokens listed in `metadata.json`, checked on
screen (compact, hovered, a long label) with `morc` and a screenshot.
