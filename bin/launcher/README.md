# bin/launcher: what `npx morceau` runs before the engine

`bin/morc.js` is the entry (the `morc` and `morceau` bins). It only routes; each job is its own file here.

| file | job |
|---|---|
| `install.js` | the first-run installer: banner, **Install for Windows / Install for Mac**, downloads that platform's engine from the GitHub release `v<version>`, copies the app files out of the npm package, makes the data folder, checks Python, runs `morc init`. Picking the other OS says to run it on that machine and whether its build is on GitHub. |
| `welcome.js` | `npx morceau` with no arguments once installed: version, running or not, six commands with one line each. |
| `menu.js` | arrow-key pick-one menu (number keys, Enter, q/Esc); a typed number off a TTY. |
| `fetch.js` | HTTPS download with redirects and a progress bar; `exists(url)`. |
| `places.js` | runtime, bin, app and data folders (mirrors `src/data/paths.rs` and `tools/cli/_home.py`). |
| `style.js` | chalk colors with a plain-ANSI fallback; no color off a TTY or with `NO_COLOR`. |

**Split from GitHub.** The npm package carries no engine. Each platform's engine is a GitHub release asset named in
`scripts/npm/platforms.js` (`morc-windows-x64.exe`, `morc-macos-arm64`, `morc-macos-x64`); the release must exist for the package version
before `npm publish` (`scripts/npm/check-publish.js` checks it).

**Never clobbers a setup.** An existing `app.txt` / `home.txt` is kept; an engine the installer did not install is kept as `morc.exe.bak`.
