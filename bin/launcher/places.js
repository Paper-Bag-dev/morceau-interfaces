// Where an installed Morceau lives. Mirrors src/data/paths.rs and tools/cli/_home.py so the launcher, the engine and the plugins agree:
//   runtime   %LOCALAPPDATA%\morc (Windows) or ~/Library/Application Support/morc (macOS): token, logs, and the install below
//   bin       <runtime>/bin/morc(.exe)   the engine, downloaded from the GitHub release
//   app       <runtime>/app              the shipped UI, plugins and docs, copied out of the npm package so they outlive the npx cache
//   data      <runtime>/data             the user's own prefs, cards and cache (remembered in <runtime>/home.txt)
const fs = require('fs');
const os = require('os');
const path = require('path');

function runtimeFor(osName) {
  if (osName === 'darwin') return path.join(os.homedir(), 'Library', 'Application Support', 'morc');
  return path.join(process.env.LOCALAPPDATA || os.homedir(), 'morc');
}

const runtime = runtimeFor(process.platform);
const exeName = process.platform === 'win32' ? 'morc.exe' : 'morc';
const read = (f) => { try { return fs.readFileSync(f, 'utf8').replace(/^﻿/, '').trim(); } catch { return ''; } };

module.exports = {
  runtime,
  bin: path.join(runtime, 'bin', exeName),
  app: path.join(runtime, 'app'),
  data: path.join(runtime, 'data'),
  appTxt: path.join(runtime, 'app.txt'),
  homeTxt: path.join(runtime, 'home.txt'),
  installedFile: path.join(runtime, 'installed.json'),
  read,
  /** {version, platform} written by the installer, or null for a machine that never ran it. */
  installed() { try { return JSON.parse(read(path.join(runtime, 'installed.json'))); } catch { return null; } },
};
