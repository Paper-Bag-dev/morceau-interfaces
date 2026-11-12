---
name: morc-build
description: Build or change Morceau components, views, chips and plugins (Preact + Tailwind, the look preset, named actions, layout rules) and check them with the card tree. Use when a card needs a component that does not exist yet, when a chip looks wrong (cut off, clipped, too wide), or when the user wants a card styled or behaving differently.
---

# Building for Morceau

Read first: `docs/views.md` (Preact, Tailwind, the preset, layout rules), `docs/state.md` (stores, named actions), `docs/plugins.md`,
`docs/chips/INDEX.md` (chips and the ladder), `docs/trees.md` (the check).

1. **Data before code.** A new look, size or behaviour is a chip JSON (`cache/seed/`), a card JSON (`morc card NAME --chip ... --set ...`),
   `opts.actions`, a preset variable or a `class`, before it is a component. Tuning values go in `ui/limits.json` / `prefs/` / `--ctl-*`.
2. **Layouts on the fly**: the `el` component (`{"type":"el","tag":"div","class":"flex gap-2","children":[...]}`) with Tailwind classes; the
   preset maps the theme (`bg-card text-fg text-muted ring-accent rounded-card`). `hover:` works on macOS too (it means `:is(:hover,.hov)`).
3. **A real component**: Preact + htm, no build step:
   ```js
   import { registerView, html } from '../core/view.js';
   registerView('name', ({ c, useBind, act }) => html`<div class="flex flex-col gap-1.5 w-full">...</div>`);
   ```
   Put it in the user's `<home>/plugins/` (outside the repo) or `ui/components/<name>/` (with metadata.json, `morc ui build`). Reload it in open
   cards with `morc refresh <name>`: data is kept, nothing pops.
4. **Layout rules (drawn right the first time):** the card owns its width (the engine locks it; or `w-[...]` on the root), children `w-full`,
   text `min-w-0 break-words`, lists `max-h-[...] overflow-y-auto`, controls `shrink-0`, no fixed widths or heights on content.
5. **Behaviour as named actions** (`opts.actions`): steps of `set`, `push`, `pull`, `merge`, `inc`, `call`, with `$`, `$.key`, `{$.key}`,
   `$id`, `$now`. Idempotent where possible. `opts.persist` keeps data across restarts.
6. **Check with the tree**: `morc tree <card>` must show no `cut`, `clipped`, `offcard`, `empty`, and `"width"` `locked` or `spec`. Fix the chip
   or component, document the fix in its md, never work around it per card.
7. **Engine changes** (Rust, ui/core) are cross-platform in the same change: `cargo check --target x86_64-pc-windows-msvc` before done.
