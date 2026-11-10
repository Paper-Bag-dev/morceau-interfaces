# scripts/

One folder per platform, plus the npm packaging. Platform-specific build steps live only in their own folder.

| folder | what |
|---|---|
| `windows/` | `build.cmd` (build + install to `%LOCALAPPDATA%\morc\bin`), `release-build.ps1` (clean build into `bin\native\windows-x64`) |
| `macos/` | `build.sh` (build + install to `~/.local/bin/morc`), `release-build.sh [target]` (clean build into `bin/native/macos-<arch>`) |
| `npm/` | `platforms.js` (the list of builds: keys, release asset names, Rust targets), `build-from-source.js` (the `prepare` step), `check-publish.js` (the `prepublishOnly` guard) |

Releases are built by CI: push a tag `v<version>` and `.github/workflows/release.yml` builds every platform, attaches the binaries to a
GitHub release and publishes the npm package with all of them inside.