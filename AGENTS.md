# AGENTS.md: working on Morceau (for any coding agent)

Morceau is a Windows-only host that shows borderless, transparent, self-expiring UI windows ("surfaces") driven by an AI agent through a CLI (`morc`).
Rust engine (tao + wry/WebView2 + the `windows` crate) + an ES-module web UI served from disk + CLI plugins (Python). Read `README.md` first.

## Layout (what lives where)
| path | what |
|---|---|
| `src/` | the Rust engine, one folder per responsibility (`cli/ host/ placement/ surface/ agent/ learning/ ipc/ data/ platform/`); each folder has a README with a table of its files; start at `src/README.md` |
| `ui/` | the page runtime served to every WebView: `core/` (store, renderer, bridge, motion, theme), `components/`, `charts/`, `models/` (3D), `plugins/`, `vendor/` |
| `tools/cli/*.py` | CLI plugins: any script here becomes `morc <name>`; files starting with `_` are helpers. `tools/graph_export.py` feeds `morc graph` |
| `cache/seed/`, `cache/fragments/`, `cache/recipes/` | shipped generic components, reusable sub-trees, example recipes |
| `docs/` | protocol, design tokens, charts, graphs, extending, the agent system prompt (`agent.md`) and the chore prompt |
| `examples/`, `integrations/` | sample triggers/actions/data; Claude Code hooks and skill |
| `bin/` | `morc.js` (npm shim); `morc.exe` is a build artefact (git-ignored) |

Three folders at runtime (see `src/paths.rs`): the install (`MORC_APP`), the user's data (`MORC_HOME`, or `<runtime>/home.txt`), and per-machine
runtime state (`%LOCALAPPDATA%\morc`). Never put user data in the install; never write secrets anywhere tracked.

## Build, test, run
- Build: `cargo build --release` (binary `target/release/morc.exe`; `target/release/morc` on macOS). **`cargo test` does NOT rebuild the exe**; always `cargo build --release` after a change
  and restart the host (`morc stop`) so a stale exe or stale UI JS is not what you test.
- Tests: `cargo test --release` (lifetime, lint). UI files are plain ES modules: check each with `node --check <file>`.
- Run: `morc use ask-permission` (cold start spawns the host detached; the first call can take a while). `MORC_DEBUG=1` adds placement/layout logging to `host.log`.
- A UI change needs a restart of the host (the warm spare page caches JS). Verify visually: take a screenshot (process must be DPI-aware or it crops).

## macOS port (two platforms, one tree)
Windows and macOS are both maintained from the same source; nothing Windows-only was removed. Read this before touching `src/`.
- **How the split works.** Windows code stays where it was, behind `#[cfg(windows)]`. The macOS code sits right beside it behind
  `#[cfg(target_os = "macos")]` (or `#[cfg(not(windows))]` for plain Unix things like paths and the clock). Cargo has the `windows` crate under
  `[target.'cfg(windows)'.dependencies]` and the macOS crates (`objc2`, `global-hotkey`) under a macOS target section.
- **Where the macOS code is.**
  - `src/platform/macos/`: the same four modules as `src/platform/` (`window_style`, `backdrop`, `hotkeys`, `foreground_watch`) with the same function
    signatures, plus `sys.rs` (every raw AppKit / CoreGraphics / libdispatch call in one place). `platform/mod.rs` picks one set per OS, so `main.rs` and
    the host are unchanged.
  - Shared files that got a macOS branch beside the Windows one: `placement/desktop.rs` (`mod mac` at the bottom), `placement/rect.rs`,
    `surface/mod.rs` (window builder, start URL, native handle), `host/mod.rs` (Accessory activation policy, mouse button, beep), `agent/worker.rs`
    (idle time), `agent/wake/process.rs` (PATH separator), `cli/connect.rs` (detached host start), `cli/plugins.rs` and `cli/mod.rs` (`util::PYTHON`),
    `data/paths.rs` (runtime folder), `learning/annoyance.rs` and `util.rs` (local time), `tools/cli/_home.py` and `tools/cli/tracker.py` (paths, exe name).
- **What differs on macOS.**
  - Runtime folder: `~/Library/Application Support/morc` (Windows: `%LOCALAPPDATA%\morc`). The binary is `morc`, not `morc.exe`. Python is `python3`.
  - Page URL: `morc://localhost/index.html` (WKWebView) instead of `http://morc.localhost/index.html` (WebView2). `assets::handle` only reads the path.
  - Handles: `Surface::hwnd()` keeps its name but returns the NSWindow pointer on macOS; the platform functions take it as `isize` on both systems.
  - Click-through: macOS has no window regions. `window_style::set_region` stores each card's content rects, and a 40 ms timer on the main thread flips
    `ignoresMouseEvents` depending on whether the pointer is over content (`wants_mouse`). AppKit calls must run on the main thread (`sys::on_main`).
  - Placement reads the window server (`CGWindowListCopyWindowInfo`, safe on any thread) and `NSScreen` (main thread). Points are converted to physical
    pixels with the primary display's scale factor, the same space tao's `outer_position` uses. Mixed-DPI multi-monitor setups are approximate.
  - The active-window watch polls the window server every 400 ms (Windows uses an event hook).
  - No permission prompts by design. Not available until we ask for Screen Recording: backdrop sampling (`backdrop::average_rgb` returns None, so cards
    keep their own scheme) and window titles (`fg_title` returns None, so the chore agent's activity log has app names only).
  - No system-wide move cursor while carrying a card (`drag_cursor_on/off` are no-ops). WKWebView uses the GPU, so `MORC_WV_ARGS` is Windows-only.
  - Hotkeys: Ctrl+Option+X and Ctrl+Option+Space through `global-hotkey`. The second clashes with macOS "next input source" when several keyboard
    layouts are on; the CLI (`morc summon`) always works.
- **Known gaps (see BACKLOG.md):**
  - Clicking a card activates morc (a plain NSWindow cannot be a non-activating panel). Fixing it means turning the window into an NSPanel.
  - The npm launcher (`bin/morc.js`), `build.cmd`, `scripts/release-build.ps1` and CI are still Windows-only. On a Mac, build with `cargo build --release`
    and run `target/release/morc`.
- **Also changed on this branch (both platforms):**
  - Every card sits on ONE plate by default: `ui/core/render.js` gives a layout root (column, row, panel) the `.card-plate` class, styled in
    `ui/components/plate/plate.css`; pieces inside go flat, and an urgent ring hugs the plate. `"plate": false` on the root opts out. `layout` now
    depends on `plate` so the material always loads.
  - Placement: a maximised active window (no free side in either tier) now uses the screen thirds, the same rung as fullscreen, instead of the
    scored corner fallback (`placement/regions.rs`, `compute`).
  - Every card goes through the tier ladder (tier 0, tier 1, then the thirds, right third first) unless its anchor is in
    `prefs/placement.json` "explicit_anchors" (bottom/corner intents). Before, a card with the default `near_focus` anchor skipped the ladder
    and used scored placement, which put cards bottom-left over a maximised window (`host/layout.rs`). The pre-measure wrap width follows the
    same rule (`host/view.rs`), and when nothing in the ladder fits, a card starts top-right of the work area instead of the scorer's corner.
    Precedence: `--at`, then a region the card names, then the spot the user carried that topic to, then the ladder.
  - Background agent modes (`prefs/worker.json` "enabled"): `"no_session"` (the default) runs it only while no interactive Claude session is
    live, so with a session open every click goes to that session; `true` = any time; `false` = never (`agent/worker.rs` `Mode`, `wake/gate.rs`).
    `morc hook` hands every click to the session, including `bg:true` ones, unless the agent was allowed to run when the click happened
    (`agent/inbox.rs`). A session only counts as live through the Claude Code hooks (`integrations/claude-code/`).
- **Paste blocks (both platforms):** the `pastes` component (`ui/components/paste/`) and the `paste-blocks` chip hand over text with a Copy
  button. The `copy` action now also goes to the host, which copies with `pbcopy` on macOS (`surface/actions.rs`); the `open` action uses
  `open` on macOS instead of `rundll32`.
- **Cheaper cards (three tiers):** (1) fixed follow-ups run straight from the button: `morc stage` + the `confirm-run` chip (no model);
  (2) small follow-ups go to the background agent with `"bg": true` and a `task`; it does the simple ones and hands the rest to the session
  with `morc escalate` (guards in docs/agent.md "Hand over to the session"); in "no_session" mode a routed (`bg`) click reaches it even while
  a session is live; (3) decisions for the session batch on one `ask-many` card. `morc session` marks a session live without the hooks.
- `paths::app_dir` resolves links before looking for the install, so a `morc` link in a bin folder (or an npm global bin) finds its plugins.
- **Before pushing a platform change:** build on both systems (`cargo build --release` on Windows and on macOS), or at least `cargo check` for the other
  target. A cfg mistake only shows up when that target is compiled.

## Rules of the house
- **Show on screen first (hard rule).** Whenever the user asks to be shown something (list, status, report, plan, progress), first check whether it can go on their screen through the `morc` CLI and do that. Terminal text only if morc is unavailable or the host defers, and say why. Text may accompany a card, never replace it.
1. **Composable, not edited per variation.** A new look, size, model or card is data (fragment params, recipe `vars`, `--set` / `--data` flags), never a
   code edit. If you find yourself editing a CLI tool or Rust to change one widget, stop and make the mechanism generic instead.
2. **Data-driven policy.** Placement weights and anchors live in `prefs/placement.json` (hot-reloaded), the chore agent in `prefs/worker.json`, taste in
   `prefs/annoyance.json`. Add a knob to the file, not a constant to the code.
3. **The host does the geometry.** Agents name a region (`--region`, `--align`, `--dx/--dy`); they do not compute pixels. The host divides the screen
   (`src/regions.rs`) and falls back to scored placement (`src/placement.rs`).
4. **Specs must pass the linter** (`src/lint.rs`): no dead controls, answers must resolve, icon-only controls need a title, every number has a unit.
   Actions only from `actions.json`; never put a shell command in a spec.
5. **Design tokens are law** (`docs/design.md`): one accent, rounded, small, sentence case, no all-caps/eyebrows/mono labels/arrow glyphs, type floors.
6. **Keep personal data out of this repo.** No tokens, paths from your machine, names, stats files. Examples use `myapp`/`stats.json`. The `main` branch is
   the shareable engine; personal setups live on a separate branch or in the data folder.
7. **Responsive by default.** Padding plus flex/flow; no absolute positioning for content; components size themselves from their content and the box (labels measured, long text ellipsised); the window follows the content and hover growth is a smooth transition. A long label, a narrow box or a hover must not clip anything. **Never clip, always size within the container:** an item is exactly as wide as its content, never padded to a default width; text wraps or ends in an ellipsis.
8. **No hardcoded sizes (strict).** Choices like min widths, radii, type sizes, paddings and text floors are tokens in `ui/tokens.json` (override in `prefs/ui.json`), sent as CSS variables; never a number typed into `theme.css` or chart code.
8b. **Modular, JSON-controlled (strict).** Any component you make: layout in one JSON, every variable thing (data, align, importance, floors, region, ttl) a field of its card JSON, never per-component code in Python or Rust; docs/chips/INDEX.md has the full rule and the done-checklist.
9a. **Split responsibilities into folders (strict, default).** Every isolated unit lives in its own folder with its code, styles, metadata.json and a short md: UI components are `ui/components/<name>/` (`morc ui new`, `morc ui build`, index `ui/COMPONENTS.md`), Rust is one module per responsibility (split `host.rs` by concern, never grow it), config is one small JSON per component. Dependencies are listed by name, never copied or merged into a big file. Start by reading the index and fetch only the folder you need.
9. **Follow the ladder.** Putting cards on screen always goes through THE LADDER in `docs/chips/INDEX.md` (need, look, count before pushing, group by kind, assign from the quadrant edge, a fixed order of what to give up, small batches, verify, fix in JSON). Never skip a rung.
10. **Cheap by default.** Animated 3D defaults to `quality: lite` and a capped fps; software GL is the default backend. Measure CPU before leaving anything animated running.

Putting something on screen = **use a chip** (`docs/chips/INDEX.md`: pick the chip, edit its data JSON, `morc use <chip> --data @file`), never a freshly written spec. Read `docs/learnings.md` before touching charts, placement or the wake path: it records each failure and the rule it became (for example, every chart kind
exports `width()` so a card is exactly as wide as its content).

## Pitfalls we already hit
- PowerShell: `R`, `rp`, `diff`, `sc`, `sleep` and friends are aliases; never name a helper function with a short alias-like name. `@(@(a,b))` with one pair
  flattens. A backtick-n inside single quotes is a literal. Prefer the editor tool or a small script file over inline replacement.
- Windows arguments mangle inline JSON: pass JSON by file (`morc compose --recipe FILE`).
- Files written by PowerShell carry a BOM: every JSON reader here tolerates it (`utf-8-sig`, `trim_start_matches('\u{feff}')`).
- WebGL hardware contexts get reset on some machines: SwiftShader is the default (`MORC_WV_ARGS=gpu` opts out). `backdrop-filter` on a WebGL surface is a risk.
- The window shape is the union of `[data-hit]` rects; a stray full-size container with `data-hit` makes a rectangle.
- **Never write a literal `\n` into a file.** Scripted edits (PowerShell string replace, here-strings with escapes) have slipped `\n` into JS, Rust and docs and broken builds. Do multi-line edits with the editor tool (Edit/Write), not a replace script. If you must script one, use a here-string or real newlines, then check `git diff -U0 | Select-String '^\+.*\\n'` before committing and run `node --check` / `cargo build`.
