#!/usr/bin/env node
// Entry for `npx morceau` and the global `morc`. The pieces live in bin/launcher/ (README there):
//   npx morceau                 first time: the installer (Install for Windows / Install for Mac); after that: a short welcome
//   npx morceau install [--platform windows|mac|macos-arm64|macos-x64]    install or update (setup is an alias)
//   npx morceau <command> ...   runs the engine: the installed one in the runtime folder, else a build in bin/native/ (a dev checkout)
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { detect } = require('../scripts/npm/platforms.js');
const places = require('./launcher/places.js');
const { install } = require('./launcher/install.js');
const { welcome } = require('./launcher/welcome.js');
const s = require('./launcher/style.js');

const root = path.resolve(__dirname, '..');
const version = require(path.join(root, 'package.json')).version;
const prefix = process.env.npm_command === 'exec' ? 'npx morceau' : 'morc';

function takeFlag(args, name) {
  const i = args.indexOf(name);
  if (i < 0) return null;
  const v = args[i + 1];
  args.splice(i, 2);
  return v || null;
}

/** The engine to run and whether it needs MORC_APP: the installed one, else a local build (dev checkout, run from the repo). */
function engine() {
  if (fs.existsSync(places.bin)) return { exe: places.bin, app: places.read(places.appTxt) ? null : root };
  const here = detect();
  const local = here && path.join(__dirname, 'native', here.key, here.exe);
  if (local && fs.existsSync(local)) return { exe: local, app: root };
  return null;
}

(async () => {
  const args = process.argv.slice(2);
  const want = takeFlag(args, '--platform');
  const cmd = args[0];

  if (cmd === 'install' || cmd === 'setup') process.exit(await install(root, version, want));
  if (cmd === '-v' || cmd === '--version' || cmd === 'version') { console.log(version); return; }

  const e = engine();
  if (!cmd) {
    if (e) return welcome(version, prefix);
    process.exit(await install(root, version, want));
  }
  if (!e) {
    console.log(`\n  ${s.bold('Morceau is not installed yet.')} Run ${s.cmd(prefix === 'morc' ? 'morc install' : 'npx morceau')} and pick Install for Windows or Install for Mac.\n`);
    process.exit(1);
  }
  const env = { ...process.env, MORC_BIN: e.exe };
  if (e.app) env.MORC_APP = e.app;
  const r = spawnSync(e.exe, args, { stdio: 'inherit', env });
  process.exit(r.status === null ? 1 : r.status);
})();
