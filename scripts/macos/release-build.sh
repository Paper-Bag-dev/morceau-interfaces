#!/bin/sh
# macOS: build a release engine with no machine-specific paths inside into bin/native/macos-<arch>.
# Usage: sh scripts/macos/release-build.sh [aarch64-apple-darwin|x86_64-apple-darwin]   (default: this Mac's architecture)
set -e
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
TARGET="${1:-}"
if [ -z "$TARGET" ]; then case "$(uname -m)" in arm64) TARGET=aarch64-apple-darwin;; *) TARGET=x86_64-apple-darwin;; esac; fi
case "$TARGET" in aarch64-*) KEY=macos-arm64;; *) KEY=macos-x64;; esac
rustup target add "$TARGET" >/dev/null 2>&1 || true
RUSTFLAGS="--remap-path-prefix=$HOME=~ --remap-path-prefix=$ROOT=." cargo build --release --target "$TARGET"
mkdir -p "bin/native/$KEY"
cp "${CARGO_TARGET_DIR:-target}/$TARGET/release/morc" "bin/native/$KEY/morc"
chmod +x "bin/native/$KEY/morc"
echo "built bin/native/$KEY/morc"
