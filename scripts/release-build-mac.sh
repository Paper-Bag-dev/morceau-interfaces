#!/bin/sh
# Build the macOS engines for the npm package with no machine-specific paths inside (use this for anything you publish).
# Usage: sh scripts/release-build-mac.sh      -> bin/morc-darwin-arm64 (+ bin/morc-darwin-x64 if the x86_64-apple-darwin target is installed)
set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
export RUSTFLAGS="--remap-path-prefix=$HOME=~ --remap-path-prefix=$root=."
mkdir -p bin
build() {   # $1 = rust target, $2 = file name in bin/
  cargo build --release --target "$1"
  cp "target/$1/release/morc" "bin/$2"
  chmod 755 "bin/$2"
  if grep -q "$USER" "bin/$2"; then echo "WARNING: bin/$2 still contains your user name"; else echo "built bin/$2 (no personal paths inside)"; fi
}
build aarch64-apple-darwin morc-darwin-arm64
if rustup target list --installed | grep -q x86_64-apple-darwin; then build x86_64-apple-darwin morc-darwin-x64; else echo "skipped Intel: rustup target add x86_64-apple-darwin"; fi
