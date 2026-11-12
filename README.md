# Morceau

**Ephemeral, agent-driven UI for Windows and macOS.** An AI agent (or you) shows small, glanceable windows on your screen with one CLI command: a stat card, a chart,
a form, a yes/no ask, a graph, a 3D model. They are borderless and transparent, sit next to what you are doing without covering it, can be clicked and
dragged, and then quietly fade away. The engine is a small Rust host; everything it draws is plain web UI served from disk.

```
 agent / script / you
        |  morc use stat-card-min --data ...        (one short CLI call, cached components, ~80 tokens)
        v
   morc CLI  --localhost, token-checked-->  Rust host (windows, placement, lifetime, linter)
                                                  |  one window per card (WebView2 / WKWebView), all in ONE web process
                                                  v
                                            ui/ (components, charts, 3D) --clicks, drags--> inbox --> agent (via hooks)
```

## What you get
- **A tiny CLI** (`morc use | show | compose | patch | close ...`) and a cache, so showing something you showed before costs one line.
- **Components**: text, stat (hero numeral with a needle), button, progress, input, list, chart (line, area, bar, donut, radar, heatmap, clock, rings, treemap,
  gauge, spark), graph (Cytoscape), 3D model viewer (three.js: procedural or glTF with animation or a bone-driven procedural idle), skeleton, spinner.
- **Placement that does the geometry for you**: the host divides the free screen around the window you are working in into regions (right, below, left, above,
  the four corners). You say `--region right --align top`; it fits, or falls back to the least-overlapping spot.
- **Fragments and recipes**: small reusable sub-trees composed into widgets, cached by structure, parameterised with `{{params}}` and `--set` / `--data` flags.
- **A linter** that refuses dead controls, questions that never resolve, icon-only buttons without a title, and numbers without units.
- **A preference vector** ("annoyance vector") the agent edits when you complain (too big, too often, too flashy), per topic, with a log of why.
- **Triggers** (zero tokens): JSON rules that watch a number in a file and show a cached widget, plus an optional gated **chore agent** (a headless low-cost model
  that wakes only when you are at the PC and no interactive session is open, and sets its own next wake).
- **Agent integration**: Claude Code hooks and a skill, `AGENTS.md` / `CLAUDE.md` for any coding agent, and a generic system prompt (`docs/agent.md`).

- **Trees instead of screenshots** (`morc tree`): the scene (displays, windows front to back, cards, what covers what) and each card's own
  tree (components, rects, and flags such as `cut`, `clipped`, `empty`), so an agent can check a card without taking a screenshot.
- **Cards as resources** (`morc card <id> get|set|hide|show|front|lower|raise|move|size`, `morc cards`): hide one for a while, bring it
  back where it was, make it big and centred, send it under the active window. Nothing pops twice; a closed card comes back with its data.
- **State**: named actions per card (`morc do`), a store shared by every card (`@/path`), snapshots and an action history on disk.
- **MCP**: `morc mcp`, seven fixed tools over the same commands.
- **Your own look and components**: Tailwind classes written on the fly (`"class": "..."`), Preact components, a look preset you own
  (`ui/preset.css`), plugins in your data folder, `morc refresh` reloads them in open cards without losing their data.
- **Hands-on**: cards snap to windows, other cards and screen edges while you carry them (never during a fast throw), and step through
  the window stack with the arrow keys. What changed and when: `CHANGELOG.md`.

## Requirements
- Windows 10 or 11 (x64) with the **Microsoft Edge WebView2 Runtime** (preinstalled on Windows 11 and current Windows 10).
- or macOS (Apple silicon; Intel when the package carries `morc-darwin-x64`): nothing else to install (WKWebView is part of macOS).
  Window titles in the scene tree need no permission (only app names are read).
- **Node 16+** only for the npm install path. **Python 3.8+ (`py` launcher)** for the extended commands (`compose`, `cache`, `init`, `place`, `wake`, ...).
- To build from source: the Rust toolchain (https://rustup.rs) and the MSVC build tools (Windows) or `xcode-select --install` (macOS).

## Install
**From npm (prebuilt, recommended; Windows and macOS)**
```sh
npm i -g morceau          # adds `morc` (and `morceau`) to your PATH
morc init                 # creates your data folder, caches the shipped components
morc use ask-permission   # your first widget
```
or without installing: `npx morceau use ask-permission`.

**From the git repository** (builds the engine for you; needs Rust)
```powershell
npm i -g github:Paper-Bag-dev/morceau-interfaces
```

**From source**
```powershell
git clone https://github.com/Paper-Bag-dev/morceau-interfaces.git
cd morceau-interfaces
cargo build --release                 # target\release\morc.exe (macOS: target/release/morc)
.\target\release\morc.exe init
```
`build.cmd` builds and installs to `%LOCALAPPDATA%\morc\bin\morc.exe`. Put that folder on your PATH, or use the npm install.

## Full setup, step by step
1. **Install** as above, then run `morc status`. The first command starts the host (hidden, detached) in about 3 seconds. `morc stop` ends it.
2. **`morc init`** creates your data folder (see below), copies `actions.json` from `examples/`, and caches the shipped seeds and recipes. It is safe to
   repeat. `morc init --home D:\my-morceau-data` keeps your data somewhere of your choosing and remembers it (no environment variable needed).
3. **See it work**: `morc use ask-permission`, `morc use bike-viewer`, `morc graph --root tools/ --langs python`. Click, drag (hold ~0.1 s), pin, dismiss (the x).
4. **Make your own**: `morc commands` lists the tools; `docs/extending.md` shows components, fragments, recipes and CLI plugins; `docs/protocol.md` is the spec format.
5. **Wire up your agent** (optional): see *Agents* below.
6. **Triggers** (optional): copy an example from `examples/triggers/` into `<data folder>\triggers\` and point `source.file` at a JSON file you maintain.
7. **Chore agent** (on only while no Claude session is open, by default): `"enabled"` in `<data folder>\prefs\worker.json` is `"no_session"` (the
   default), `true` (any time) or `false` (never). Pick `model`, and read the gates below first.

## Your data folder
Everything that is yours lives outside the install, so upgrading never touches it. It is found by, in order: `MORC_HOME`, the path in
`%LOCALAPPDATA%\morc\home.txt` (written by `morc init --home`), or the install folder itself (portable mode).

| path | what |
|---|---|
| `prefs/annoyance.json`, `annoyance_log.md` | the preference vector and a log of who changed what and why |
| `prefs/placement.json` | placement weights, thresholds and anchor shapes (hot-reloaded: edit with `morc place`) |
| `prefs/worker.json` | the chore agent: enabled, model, idle seconds, rate limits, title tracking and redaction |
| `triggers/*.json` | rules that show a widget (or wake the chore agent) when a number in a file crosses a threshold |
| `actions.json` | the allowlist of host-side commands a widget may *name* (a spec can never carry a command) |
| `cache/components/` | saved specs; `cache/recipes/`, `cache/fragments/` are yours too (they shadow the shipped ones) |
| `tools/cli/` | your own CLI plugins (they shadow shipped ones of the same name) |
| `docs/agent.md`, `chore-prompt.md` | optional overrides of the agent system prompt and the chore wake prompt |

Per-machine runtime state (token, logs, usage counts, the installed exe) is under `%LOCALAPPDATA%\morc`. The install folder is `MORC_APP` or found from the exe.

## Shortcuts
**Keyboard** (system-wide; the arrow keys alone only while the pointer is on a card)
| Windows | macOS | what it does |
|---|---|---|
| `Ctrl+Alt+X` | `Ctrl+Option+X` | dismiss every card at once |
| `Ctrl+Alt+Space` | `Ctrl+Option+Space` | bring back anything that needed you (parked urgent asks), next to the active window |
| `Ctrl+Alt+Up` | `Ctrl+Option+Up` | every card to the top of the window stack |
| `Ctrl+Alt+Down` | `Ctrl+Option+Down` | every card to the bottom of the stack (under every window that overlaps it) |
| `Up` / `Down` with the pointer on a card | same | step THAT card one window up / down the stack, as far as it can go |

If a combination is already taken (some graphics drivers use `Ctrl+Alt+arrows` to rotate the screen; macOS uses `Ctrl+Option+Space` for the
next input source when several keyboard layouts are on), it is quietly skipped (host.log says so) and the command still works:
`morc dismiss-all`, `morc summon`, `morc card <id> raise|lower`. No permission prompt on either system; the arrows are taken only while
the pointer is on a card and handed back the moment it leaves.

**On a card (mouse), the same on Windows and macOS**
| gesture | what it does |
|---|---|
| hold ~0.1 s on empty space or text (0.2 s on buttons), then drag | carry the card; near a window, another card or the screen edge it snaps on (keeping a small seam, `snap-gap`), slides along it, and lets go when you pull away; a fast throw never snaps. The spot is remembered for that topic |
| move to the card's top-right corner | its three buttons come out on an arc around the corner (only there, not on the whole card) |
| pin (first button) | keep the card open; hold it to carry the card |
| x (second) | dismiss this card |
| layer (third) | send the card one level down the window stack; click again to bring it back on top |
| right-click | dismiss this card |
| hover | pauses the lifetime; a compact card springs open |
| click anything | extends the lifetime and teaches the annoyance vector what you engage with |

Snap distances, button size and spacing are data, not code: `ui/limits.json` (`snap-px`, `snap-break-px`, `snap-gap`, `snap-max-speed`) and the
`--ctl-*` variables (`ui/preset.css`).

**Command line (the ones used every day)**
```powershell
morc card NAME --chip CHIP --set key=value ...   # write or change one card's data file (remembered in cards/)
morc push NAME [NAME ...]                        # show cards; the host places them in free space (or --all --prefix p-)
morc status                                      # the cards on screen: id, rect, visible
morc space   |  morc free                        # the free regions around the active window
morc patch ID /path value                        # change one value live; /__pulse true plays the blue look-here pulse
morc ui build | list | show NAME | new NAME      # the UI component tree (folders: code + styles + metadata)
morc annoy nudge crowd 0.8 --why "too crowded"   # teach the host what you treat as crowded or too big
morc commands                                    # every CLI plugin with one line each
morc dismiss-all   |  morc summon                # the hotkeys, as commands
morc tree [ID] [--all] [--depth N]               # the scene tree / one card's tree / both merged (flags instead of screenshots)
morc cards   |  morc card ID get                 # every card the host knows / one card's state (shown, hidden, closed, layer, spot)
morc card ID hide|show|front|lower|raise         # put away, bring back, front, one step down / up the window stack
morc card ID move --at X,Y|--center|--snap       # move it (glides); --snap pulls it onto the nearest edge
morc card ID set '{"frac":0.9,"center":true}'    # the state you want, in one go (state, layer/depth, at, center, frac)
morc do ID ACTION '{"text":"..."}'               # run a card's named action
morc patch @shared /focus '"..."'                # the store every card can bind to ("@/focus")
morc refresh [COMPONENT]                         # reload components / plugins in the open cards, data kept
morc mcp                                         # the same, as an MCP server (stdio)
```
## Ten-minute tour
```powershell
morc use ask-permission --data @ask.json            # inline JSON loses its quotes in Windows shells: pass a file (@file.json)
morc show spec.json                          # any spec: {id?, opts?, components, data?, sources?}
morc save spec.json                          # cache it as a named component
morc use my-card --data @card.json --region right --align top   # place by name, the host does the geometry
morc patch my-card /n 43                     # change one value on screen
morc compose --recipe cache\recipes\stat-card-min.json --data h.label=Focus --data h.value=42 --data h.unit=min --use -- --region auto
morc free                                    # where is there room right now
morc cache stats                             # LFU: what is used, what will fade
```
A spec is JSON: a flat list of `{id, type, ...props, children?}` plus a data model. Any prop can be a literal, `{"bind":"/path"}`, or a template string `"CPU {/cpu:0}%"`.

## Concepts
- **Surface**: one window with a lifetime (default ~15 s, hover pauses, interaction extends, pin holds, sticky skips the timer, urgent is ringed and parked if ignored).
- **Component / fragment / recipe**: a spec you saved; a reusable sub-tree with `{{params}}`; a recipe that composes fragments, cached by structure.
- **Region**: a free rectangle around the active window. `morc use X --region right|below|left|above|top-right|...|auto --align ... --dx --dy`.
- **Context scope**: the topic prefix (`stats.minutes` -> `stats`). Surfaces of one scope line up; a different scope goes to the opposite side.
- **Linter**: runs on every show/save/stream. `opts.lint:"off"` exists for demos only.
- **Preference vector**: size, ttl, frequency, motion, sound, boldness, simplicity, overlap, dock, morph, harmony, quiet hours, muted apps and topics. Per topic too.

## Architecture
```
 +-------------------+        +-----------------------------+        +----------------------------------+
 | agent / script    |  CLI   | morc.exe  (CLI mode)        | TCP    | morc.exe  (host mode, detached)  |
 | hooks, triggers   |------->| parse, ensure host, send    |------->| 127.0.0.1:47831, per-machine     |
 +-------------------+        +-----------------------------+ token  | token; one event loop (tao)      |
        ^                                                            +----------------+-----------------+
        | inbox (clicks, drags, answers) via hooks / `morc inbox`                     |  Ev::{Cmd, Ipc, Foreground, Hotkey}
        |                                                                             v
 +------+---------------+     +-------------------+      +-----------------------------------------------+
 | chore agent (opt-in) |<----| wake gates        |<-----| surfaces: one WebView2 window each + a warm   |
 | claude -p, exits     |     | session / idle /  |      | spare; shaped by SetWindowRgn, no-activate    |
 +----------------------+     | rate / cooldown   |      | page: ui/ (store -> renderer -> components)   |
                              +-------------------+      +-----------------------------------------------+
```
**Processes.** The same exe is the CLI and the host. A CLI call connects to `127.0.0.1:47831` with the token in `%LOCALAPPDATA%\morc`, starting a hidden detached host
first if none is up (its std handles are made non-inheritable so a caller capturing output never waits on it). Everything the host does happens on one event loop;
helper threads (TCP server, data sources, triggers and wake flags, hotkeys, the foreground-window WinEvent hook) only post events to it.

**A surface, end to end.** `use` loads a cached spec and merges `--data`; the **linter** checks it; the host claims the warm spare WebView, sends it `init`
(spec + preference view); the page's **store** (JSON-pointer data model) and **renderer** build the DOM from the flat component list (`{bind}` and `{/path}` templates,
`when` visibility), then report the content size and the clickable `[data-hit]` rects. The host sizes the window, clips it to the union of those rects, asks
**regions** for a spot, shows it without stealing focus, and the page springs in. `patch` changes the store and the page animates the difference. Lifetime rules
(`lifetime.rs`) expire it, park it if urgent, and log ignored, glanced or engaged to the engagement log.

**Placement pipeline.** (1) `--at` wins; (2) `--region` / `--align` / `--dx` / `--dy` or `auto`: the active window is the centre of a 3x3 grid, each of the eight cells
(and the four bands) minus every window and widget is a free rectangle (tier 0), then minus only the active window (tier 1, if the `overlap` preference allows);
default alignment hugs the active window; (3) if nothing fits, scored placement over a candidate grid (overlap, toast zone, flush-to-neighbour, same-scope harmony,
opposite-side-for-other-scopes) using weights from `prefs/placement.json`. Fullscreen or no active window splits the screen into right, middle and left thirds.

**Data in, events out.** A spec's `sources` (file + JSON pointer) feed live values; `actions.json` is the only way a widget runs anything on the host (a spec names an
entry, never carries a command); clicks that need the agent become inbox events that Claude Code hooks inject into the next prompt.

**The cache.** Specs are cached by name; `compose` builds specs from fragments and caches them by structure hash; usage counts drive an LFU with one-week half-life and
a daily expiry. The three folders (install, your data, runtime) keep shipped files, your files and machine state apart (`src/paths.rs`, `tools/cli/_home.py`).

**Chore agent.** A trigger rule, a wake flag or a click asks for a wake; the gates run in order (interactive session open, PC idle, rate limit, chore cooldown); if they pass
the host spawns `claude -p` with `docs/agent.md` as the system prompt and `docs/chore-prompt.md` filled with recent window titles. The agent does at most two chores,
sets its next wake (`morc wake set`) and exits.

| extension point | where | needs a rebuild? |
|---|---|---|
| new look, size, model, card | fragment `{{params}}`, recipe `vars`, `--set` / `--data` flags | no |
| placement, taste, chore agent | `prefs/placement.json`, `annoyance.json`, `worker.json` (hot-reloaded) | no |
| new component or chart kind | a file in `ui/plugins/`, `ui/components/`, `ui/charts/` | no |
| new command | a script in `tools/cli/` (`# morc <name>: description` on the first lines) | no |
| a new engine capability (protocol message, platform layer) | `src/` | yes |

## Themes and design rules
One accent (coral by default; `morc annoy theme <name|auto>` or `opts.theme`), rounded pill shapes, small buttons, text on a frosted plate, colours that flip
against the page behind (backdrop-aware), no window chrome. Type floors 11.5 / 12.5 / 15 / 22 / 32 px, icons at least 14 px, charts at least 96 px tall. Sentence
case; no all-caps labels, eyebrows, monospace data labels or arrow glyphs. Motion only when the user triggers it or for an urgent ask. Tokens and the reasoning
are in `docs/design.md`; the CSS variables are in `ui/theme.css` and `ui/core/theme.js`.

## Agents
- **Claude Code**: merge `integrations/claude-code/settings.example.json` into `~/.claude/settings.json` (two hooks that inject your clicks and drags into the
  session as `[morc] N UI interaction(s) ...`), and copy `integrations/claude-code/skills/morc-screen` to `~/.claude/skills/`. `CLAUDE.md` imports `AGENTS.md`.
- **Any other agent**: point it at `AGENTS.md` (how to work on the repo) and `docs/agent.md` (how to drive it). The CLI is the whole interface.
- **Chore agent (headless; by default only while no Claude session is open)**: the host can spawn `claude -p --model <model>` with `docs/agent.md` as its system prompt. Gates, in order: an interactive
  session was active recently (rest), the PC has had no input for `idle_s` (rest), the hourly rate limit, the chore cooldown. It exits when done, and sets its next
  wake with `morc wake set`. By default (`"enabled": "no_session"`) it runs only while no interactive
  Claude session is live (one whose hooks touched morc within `session_fresh_s`); with a session open, your clicks go to that session instead.
  `true` lets it run any time the gates allow; `false` turns it off. Without the Claude Code hooks installed no session ever counts as live.

## Security and privacy
- The command port is `127.0.0.1` only and needs the per-machine token in `%LOCALAPPDATA%\morc`.
- Specs can only use registered components and named actions; commands run only from the `actions.json` allowlist, with fixed arguments.
- The host never captures screen content. Window-title tracking (for the chore agent) is on only if `track_titles` is true; titles containing any `redact` word are
  never stored; the log stays local, but the chore prompt sends recent titles to the model you configured, so keep that in mind.
- Downloaded 3D models are not part of this repository; each keeps its author's licence (`THIRD-PARTY.md`).

## Troubleshooting
- Logs: `%LOCALAPPDATA%\morc\host.log`; `MORC_DEBUG=1` adds placement and layout lines.
- Nothing appears: `morc status`; if the host is up, `morc dismiss-all` then `morc use ask-permission`. A stale host serves stale UI files: `morc stop`, then retry.
- WebGL context lost / black canvas: software GL (SwiftShader) is already the default; `MORC_WV_ARGS=gpu` opts into hardware GL.
- A model looks heavy: 3D defaults to `quality: lite` at a capped fps; measure CPU before leaving an animated model running.
- First start after installing a new build can pause while Windows scans the new binary.

## Layout of this repository
`src/` Rust engine - `ui/` page runtime and components - `tools/cli/` CLI plugins - `cache/` shipped seeds, fragments and recipes - `docs/` specs and prompts -
`examples/` triggers, actions, sample data - `integrations/` Claude Code hooks and skill - `bin/` npm launcher - `scripts/` build and publish checks.
See `docs/README.md` for the full map and `BACKLOG.md` for what is next (macOS is on it).

## Contributing
Read `AGENTS.md`. Keep the engine generic: no personal data, no machine paths. Changes to behaviour go through data (prefs, fragments, recipes) before code.
`cargo test --release`, `node --check` on UI files, and the CI workflow must pass.

## Licence
MIT, see `LICENSE`. Third-party notices: `THIRD-PARTY.md`.

---
Made with love by Claude + paper_bag_dev ❤️
