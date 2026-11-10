#!/usr/bin/env node
// Launcher for `npx morceau ...` and the global `morc` command. The real program is the Rust engine for one platform, kept in
// bin/native/<platform>/ (scripts/npm/platforms.js lists them). Which one runs:
//   1. `--platform <windows|mac|macos-arm64|macos-x64|windows-x64>` (anywhere in the arguments, removed before the engine sees them)
//   2. the MORC_PLATFORM environment variable
//   3. the choice saved by `morc setup` (bin/native/selected.txt)
//   4. this machine's own build
// `npx morceau setup [--platform X]` asks which build to use (or takes X), fetches it if needed and saves the choice.
// A build that is not in the package is downloaded once from the GitHub release of this version (built by .github/workflows/release.yml).
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const https = require('https');
const readline = require('readline');
const { PLATFORMS, detect, resolve } = require('../scripts/npm/platforms.js');

const root = path.resolve(__dirname, '..');
const pkg = require(path.join(root, 'package.json'));
const nativeDir = path.join(__dirname, 'native');
const savedFile = path.join(nativeDir, 'selected.txt');
const REPO = 'Paper-Bag-dev/morceau-interfaces';

function takeFlag(args, name) {
  const i = args.indexOf(name);
  if (i < 0) return null;
  const v = args[i + 1];
  args.splice(i, 2);
  return v || null;
}
const binOf = (p) => path.join(nativeDir, p.key, p.exe);
const saved = () => { try { return fs.readFileSync(savedFile, 'utf8').trim(); } catch { return ''; } };

function download(url, dest, hops = 0) {
  return new Promise((ok, fail) => {
    https.get(url, { headers: { 'User-Agent': 'morceau-launcher' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location && hops < 6) { res.resume(); return download(res.headers.location, dest, hops + 1).then(ok, fail); }
      if (res.statusCode !== 200) { res.resume(); return fail(new Error(`HTTP ${res.statusCode} for ${url}`)); }
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      const tmp = dest + '.part', out = fs.createWriteStream(tmp);
      res.pipe(out);
      out.on('finish', () => out.close(() => { fs.renameSync(tmp, dest); ok(); }));
      out.on('error', fail);
    }).on('error', fail);
  });
}

async function ensure(p) {
  const exe = binOf(p);
  if (fs.existsSync(exe)) return exe;
  const url = `https://github.com/${REPO}/releases/download/v${pkg.version}/${p.asset}`;
  console.error(`morceau: fetching the ${p.label} build (v${pkg.version}) ...`);
  try { await download(url, exe); } catch (e) {
    console.error(`morceau: could not download ${url}\n  ${e.message}\n  Build it yourself: cargo build --release, then node scripts/npm/build-from-source.js`);
    process.exit(1);
  }
  if (p.os !== 'win32') fs.chmodSync(exe, 0o755);
  return exe;
}

function ask() {
  const here = detect();
  return new Promise((ok) => {
    console.log('Which Morceau build do you want?');
    PLATFORMS.forEach((p, i) => console.log(`  ${i + 1}) ${p.label}${here && here.key === p.key ? '   (this machine)' : ''}`));
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    const def = here ? PLATFORMS.indexOf(here) + 1 : 1;
    rl.question(`Choose 1-${PLATFORMS.length} [${def}]: `, (a) => { rl.close(); ok(PLATFORMS[(parseInt(a, 10) || def) - 1] || PLATFORMS[def - 1]); });
  });
}

(async () => {
  const args = process.argv.slice(2);
  const flag = takeFlag(args, '--platform');

  if (args[0] === 'setup') {
    const p = flag ? resolve(flag) : process.stdin.isTTY ? await ask() : detect();
    if (!p) { console.error(`morceau: unknown platform "${flag}". Choose one of: ${PLATFORMS.map((x) => x.key).join(', ')}, windows, mac`); process.exit(1); }
    await ensure(p);
    fs.mkdirSync(nativeDir, { recursive: true });
    fs.writeFileSync(savedFile, p.key);
    console.log(`morceau: using the ${p.label} build. Run \`morc use ask-permission\` to see a card.`);
    if (p.os !== process.platform) console.log(`  Note: this machine is ${process.platform}; that build runs on ${p.os === 'win32' ? 'Windows' : 'macOS'}.`);
    return;
  }

  const p = resolve(flag || process.env.MORC_PLATFORM || saved() || '');
  if (!p) {
    console.error(`morceau has no build for ${process.platform}/${process.arch}. Builds: ${PLATFORMS.map((x) => x.label).join(', ')}.`);
    process.exit(1);
  }
  if (p.os !== process.platform) {
    console.error(`morceau: the ${p.label} build does not run on this machine (${process.platform}). Pick another with: npx morceau setup`);
    process.exit(1);
  }
  const exe = await ensure(p);
  const r = spawnSync(exe, args, {
    stdio: 'inherit',
    env: { ...process.env, MORC_APP: root, MORC_BIN: exe },   // the install folder (ui/, tools/, docs/) is this package; tools find the exe the same way
  });
  process.exit(r.status === null ? 1 : r.status);
})();
