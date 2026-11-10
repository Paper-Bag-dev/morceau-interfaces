#!/bin/sh
# macOS: build the release engine and install it to ~/.local/bin/morc (stops a running host first).
set -e
cd "$(dirname "$0")/../.."
BIN="$HOME/.local/bin"
mkdir -p "$BIN"
[ -x "$BIN/morc" ] && "$BIN/morc" stop >/dev/null 2>&1 || true
cargo build --release
cp target/release/morc "$BIN/morc" && echo "installed $BIN/morc (put $BIN on your PATH)"
