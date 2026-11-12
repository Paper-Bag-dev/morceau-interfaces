#!/usr/bin/env node
// Thin launcher so `npx morceau ...` and the global `morc` command work. The real program is the Rust engine for this platform:
//   Windows x64      bin/morc.exe
//   macOS arm64/x64  bin/morc-darwin-arm64 / bin/morc-darwin-x64
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const name = process.platform === 'win32' ? 'morc.exe' : process.platform === 'darwin' ? `morc-darwin-${process.arch}` : null;
if (!name) {
  console.error('morc runs on Windows 10/11 and macOS (it draws WebView2 / WKWebView windows).');
  process.exit(1);
}
const exe = path.join(__dirname, name);
if (!fs.existsSync(exe)) {
  console.error(`${name} is missing from this install (no prebuilt engine for ${process.platform}-${process.arch} in this package).\n` +
    '  From a git clone: cargo build --release   (then: node scripts/build-from-source.js)\n' +
    '  Or build from git: npm i -g github:Paper-Bag-dev/morceau-interfaces   (needs the Rust toolchain)');
  process.exit(1);
}
const r = spawnSync(exe, process.argv.slice(2), {
  stdio: 'inherit',
  env: { ...process.env, MORC_APP: root, MORC_BIN: exe },   // the install folder (ui/, tools/, docs/) is this package; tools find the exe the same way
});
process.exit(r.status === null ? 1 : r.status);
