#!/usr/bin/env node
// Thin launcher so `npx morceau ...` and the global `morc` command work. The real program is bin/morc.exe (the Rust engine).
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

if (process.platform !== 'win32') {
  console.error('morc runs on Windows 10/11 only (it draws WebView2 windows).');
  process.exit(1);
}
const root = path.resolve(__dirname, '..');
const exe = path.join(__dirname, 'morc.exe');
if (!fs.existsSync(exe)) {
  console.error('morc.exe is missing from this install.\n  Reinstall:        npm i -g morceau\n  From a git clone: cargo build --release   (then: node scripts/build-from-source.js)');
  process.exit(1);
}
const r = spawnSync(exe, process.argv.slice(2), {
  stdio: 'inherit',
  env: { ...process.env, MORC_APP: root, MORC_BIN: exe },   // the install folder (ui/, tools/, docs/) is this package; tools find the exe the same way
});
process.exit(r.status === null ? 1 : r.status);
