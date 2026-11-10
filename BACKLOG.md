# Backlog

## Next
- [ ] **macOS build, published as one version with Windows** (to use it at work). Status: the engine code is ported (branch `mac-port`; see "macOS port" in
  AGENTS.md). Windows code is untouched behind `#[cfg(windows)]`. Still open on macOS: a first build and run on a Mac; clicks activate morc (needs an
  NSPanel); backdrop sampling and window titles (need Screen Recording); a system-wide move cursor; npm launcher and build scripts for macOS (CI now builds and tests both systems);
  signing and notarisation. Original scope, honestly:
  - The UI (`ui/`), the CLI plugins (`tools/cli`), the protocol and the cache are platform-neutral. The Windows-specific part is the Rust platform layer:
    window shaping (`SetWindowRgn`), no-activate/tool-window styles and DWM (`platform/window_style.rs`), the active-window watcher (`platform/foreground_watch.rs`, `placement/desktop.rs`),
    global hotkeys (`platform/hotkeys.rs`), backdrop sampling (`platform/backdrop.rs`), input-idle and window titles (`agent/worker.rs`), cursor swapping, the `windows` crate.
  - Plan: put those behind a small `platform` trait (`windows.rs` today, `macos.rs` new, using `objc2` / Cocoa: `NSWindow` level + click-through via
    `ignoresMouseEvents` regions, `NSWorkspace` for the active app, `CGEventSource` idle time, `CGWindowListCopyWindowInfo` for window rectangles).
    `tao` and `wry` already support macOS (WKWebView instead of WebView2). The transparent, shaped window needs a per-pixel hit test instead of a region.
  - Packaging: npm `optionalDependencies` per platform (`morceau-win32-x64`, `morceau-darwin-arm64`, `morceau-darwin-x64`) behind the one `morceau` package, so
    `npx morceau` picks the right binary; CI matrix on `windows-latest` and `macos-latest`; release assets with SHA-256 for both.
  - Things that differ: no software-GL default needed (WKWebView uses the GPU), paths (`~/Library/Application Support/morc` instead of `%LOCALAPPDATA%`), the
    `py` launcher becomes `python3`, and Gatekeeper needs a signed and notarised binary for a smooth install.
  - Check on a work machine first: does the employer allow a local helper that draws overlay windows and reads window titles? Window-title logging is off
    unless `track_titles` is on, and titles matching the `redact` list are never stored.
- [ ] **Keep both platforms, and let `npx morceau` offer the choice.** Windows and macOS are both maintained (the Windows-only code already sits in
  `src/platform/`; a macOS `platform/` sits beside it). The installer asks which one you want (windows or mac, with the machine's own as the default) and
  downloads ONLY that build, in isolation, from the GitHub repo's release assets (built by CI, with SHA-256), into its own folder
  (`%LOCALAPPDATA%\morc\bin\<platform>` / `~/Library/Application Support/morc/bin/<platform>`). `npx morceau` itself stays a tiny launcher: no binary inside the
  npm tarball, so the package is small and each platform's download is isolated and verifiable. Ties in with the release-assets item below and with the
  `platform/` folder split already done.- [ ] Move the prebuilt `morc.exe` out of the npm tarball into GitHub Release assets built by CI (provenance), with the npm `postinstall` downloading and
  verifying it.
- [ ] Optional GitHub release workflow: tag -> build -> attach the exe and its hash.

## Later
- [ ] A hub link: the same protocol over a small server so a second machine can serve when the first is off, and phone surfaces share the vocabulary.
- [ ] More fragments (calendar streaks, sankey, slope charts) and a streamgraph; a themes pack beyond the single coral accent.
- [ ] `morc doctor`: one command that checks WebView2, the data folder, the host and the plugins, and says what to fix.
