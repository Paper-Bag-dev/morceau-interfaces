# Views: Preact components and Tailwind classes

The engine has no look. Colors, fonts and radii come from the **preset** (`ui/preset.css`, yours), which maps the theme's variables onto
Tailwind names: `bg-card text-fg text-muted ring-accent rounded-card border-line text-good|warn|bad`. Tailwind's own palette works too.

## The card's buttons
Pin, x and the layer button (under the x) are one stack at the top-right with one look and one size (they do not grow with the
card's zoom), taken from variables the preset can set: `--ctl-size`, `--ctl-gap`, `--ctl-bg`,
`--ctl-fg`, `--ctl-ring`, `--ctl-ring-hover`, `--ctl-blur`, `--ctl-rest` (e.g. `#stage { --ctl-size: 26px; --ctl-blur: 0px; }` in
`ui/preset.css`). The layer button sends the card one level down the window stack and back up; Up / Down while the pointer is on the
card step it further down / up (docs/cards.md).

## Classes on the fly
Any component may carry `"class": "flex gap-2 px-3 rounded-xl"`. The generic `el` component builds layouts in the card JSON itself:
```json
{"id": "h", "type": "el", "tag": "div", "class": "flex items-baseline justify-between gap-3", "children": ["title", "meta"]},
{"id": "title", "type": "el", "tag": "h3", "class": "text-[15px] font-semibold text-fg", "text": "Tasks"}
```
`el` takes `tag` (layout and text tags only: never script, iframe or form), `class`, `text` (literal, `{bind}` or template) or `children`,
and an optional `action`. Tailwind's browser build (vendored) generates the CSS at runtime and loads only on a card that uses a class;
its reset (preflight) is left out, so the older components keep their look. `hover:` means `:is(:hover, .hov)`, because on macOS the
page cannot see the pointer while our app is inactive and the host marks the hovered chain with `.hov`.

## Preact components
```js
import { registerView, html } from '../core/view.js';
registerView('tasklist', ({ c, useBind, act }) => {
  const items = useBind(c.items) || [];          // literal, {bind}, "{/x}" or "@/shared"; re-renders when it changes
  return html`<div class="flex flex-col gap-1.5 max-h-[260px] overflow-y-auto">
    ${items.map((it) => html`<div key=${it.id} class="rounded-xl px-3 py-2 bg-white/5 hover:bg-white/10 break-words"
        onClick=${() => act({ do: 'call', name: 'focus', args: { text: it.text } })}>${it.text}</div>`)}
  </div>`;
});
```
htm tagged templates, no build step (Preact 10 + hooks + htm, vendored, via the import map in `index.html`). The spec still uses JSON:
`{"id": "l", "type": "tasklist", "items": {"bind": "/items"}}`. Put clickable boxes under `data-hit` (the window shape follows them).

## Layout rules (drawn right the first time)
A card must not need a measure-and-fix loop. Write components so the browser's layout gets it right by construction:
1. **Text wraps, never cut**: `min-w-0 flex-1 break-words` on the text box; `truncate` only with the full text in a tooltip.
2. **Long content scrolls inside the card**: `max-h-[...] overflow-y-auto` on the list, never `overflow-hidden` on content.
3. **Bounds, not sizes**: `min-w-[..] max-w-[..]`, flex and gap; no fixed widths or heights on content boxes.
4. **Controls never shrink**: `shrink-0` on buttons and icons beside text.
5. **One width owner (the engine does it)**: once a card is laid out its width is locked (`morc tree <card>` shows `"width": "locked"`), so
   a data patch wraps inside the card and never widens it; new components, `morc refresh` or a new size unlock, measure and lock again.
   A spec can own it instead: `"class": "w-[380px]"` on the root. Either way children fill it (`w-full`), never their own `max-w`.
   Earlier rule text, kept for components: the card's root sets the width (`"class": "w-[380px]"` on the root column); every child fills it (`w-full`), never
   its own `max-w`. Otherwise the widest piece (a long value in a header) grows the card while a capped list stays narrow inside it.
   Short values beside a title go in a `flex-wrap` row with `min-w-0 break-words`, so a long one drops to its own line.
6. **Check with the tree, not a screenshot**: `morc tree <card>` must show no `cut`, `clipped`, `offcard` or `empty` flags (docs/trees.md).
