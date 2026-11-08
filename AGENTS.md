# AGENTS.md: working on Morceau (for any coding agent)

Morceau is a Windows-only host that shows borderless, transparent, self-expiring UI windows ("surfaces") driven by an AI agent through a CLI (`morc`).
Rust engine (tao + wry/WebView2 + the `windows` crate) + an ES-module web UI served from disk + CLI plugins (Python). Read `README.md` first.

## Layout (what lives where)
| path | what |
|---|---|
| `src/` | the Rust engine: `host/` (one file per concern, see its README), protocol, placement/regions, policy, worker (chore agent), lint, cache, CLI client |
| `ui/` | the page runtime served to every WebView: `core/` (store, renderer, bridge, motion, theme), `components/`, `charts/`, `models/` (3D), `plugins/`, `vendor/` |
| `tools/cli/*.py` | CLI plugins: any script here becomes `morc <name>`; files starting with `_` are helpers. `tools/graph_export.py` feeds `morc graph` |
| `cache/seed/`, `cache/fragments/`, `cache/recipes/` | shipped generic components, reusable sub-trees, example recipes |
| `docs/` | protocol, design tokens, charts, graphs, extending, the agent system prompt (`agent.md`) and the chore prompt |
| `examples/`, `integrations/` | sample triggers/actions/data; Claude Code hooks and skill |
| `bin/` | `morc.js` (npm shim); `morc.exe` is a build artefact (git-ignored) |

Three folders at runtime (see `src/paths.rs`): the install (`MORC_APP`), the user's data (`MORC_HOME`, or `<runtime>/home.txt`), and per-machine
runtime state (`%LOCALAPPDATA%\morc`). Never put user data in the install; never write secrets anywhere tracked.

## Build, test, run
- Build: `cargo build --release` (binary `target/release/morc.exe`). **`cargo test` does NOT rebuild the exe**; always `cargo build --release` after a change
  and restart the host (`morc stop`) so a stale exe or stale UI JS is not what you test.
- Tests: `cargo test --release` (lifetime, lint). UI files are plain ES modules: check each with `node --check <file>`.
- Run: `morc use ask-permission` (cold start spawns the host detached; the first call can take a while). `MORC_DEBUG=1` adds placement/layout logging to `host.log`.
- A UI change needs a restart of the host (the warm spare page caches JS). Verify visually: take a screenshot (process must be DPI-aware or it crops).

## Rules of the house
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
