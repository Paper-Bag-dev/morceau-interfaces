// The installer a new user sees on `npx morceau`: two choices, Install for Windows or Install for Mac. The engine for that platform
// comes from the GitHub release of this exact version (one asset per platform, see scripts/npm/platforms.js); the UI, plugins and
// docs are copied out of the npm package into the runtime folder so they outlive the npx cache; the user's data gets its own folder.
// Never overwrites a data folder or an app.txt/home.txt someone already set up (a dev checkout keeps working).
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { PLATFORMS, resolve } = require('../../scripts/npm/platforms.js');
const places = require('./places.js');
const { download, exists } = require('./fetch.js');
const { pick } = require('./menu.js');
const s = require('./style.js');

const REPO = 'Paper-Bag-dev/morceau-interfaces';
const APP_FILES = ['ui', 'tools', 'cache/seed', 'cache/fragments', 'cache/recipes', 'docs', 'examples', 'integrations',
  'AGENTS.md', 'CLAUDE.md', 'README.md', 'LICENSE', 'THIRD-PARTY.md', 'package.json'];

const assetUrl = (version, p) => `https://github.com/${REPO}/releases/download/v${version}/${p.asset}`;
const step = (msg) => console.log(`  ${s.tick()} ${msg}`);
const fail = (msg, hint) => { console.log(`  ${s.cross()} ${msg}`); if (hint) console.log(`    ${s.dim(hint)}`); return 1; };

function banner(version) {
  console.log('');
  console.log(`  ${s.accent(s.bold('morceau'))} ${s.dim('v' + version)}`);
  console.log(`  ${s.dim('Floating cards, charts and questions that your AI agent puts on your screen, and that go away on their own.')}`);
  console.log('');
}

/** "windows" | "mac" | a platform key | null (ask) -> a platform entry, or null when the user backs out. */
async function choose(want) {
  if (want) return resolve(want);
  const here = process.platform;
  const options = [
    { label: 'Install for Windows', hint: here === 'win32' ? 'this computer' : 'Windows 10/11, x64' },
    { label: 'Install for Mac', hint: here === 'darwin' ? 'this computer' : 'macOS 12+, Apple silicon or Intel' },
  ];
  console.log(`  ${s.bold('What do you want to install?')} ${s.dim('(↑/↓ then Enter)')}`);
  const i = await pick(options, here === 'darwin' ? 1 : 0);
  if (i === null) return null;
  if (i === 0) return resolve('windows');
  if (here === 'darwin') return resolve('mac');   // this Mac's own chip
  console.log('');
  console.log(`  ${s.bold('Which Mac?')}`);
  const j = await pick([{ label: 'Apple silicon', hint: 'M1, M2, M3, M4' }, { label: 'Intel' }], 0);
  return j === null ? null : resolve(j === 0 ? 'macos-arm64' : 'macos-x64');
}

function copyApp(pkgRoot) {
  fs.rmSync(places.app, { recursive: true, force: true });
  for (const rel of APP_FILES) {
    const from = path.join(pkgRoot, rel);
    if (fs.existsSync(from)) fs.cpSync(from, path.join(places.app, rel), { recursive: true });
  }
  // point the engine at this copy, unless someone already pointed it at their own checkout
  const cur = places.read(places.appTxt);
  if (!cur || path.resolve(cur) === path.resolve(places.app)) fs.writeFileSync(places.appTxt, places.app);
  return cur && path.resolve(cur) !== path.resolve(places.app) ? cur : places.app;
}

function dataFolder() {
  const cur = places.read(places.homeTxt);
  if (cur) return cur;
  fs.mkdirSync(places.data, { recursive: true });
  fs.writeFileSync(places.homeTxt, places.data);
  return places.data;
}

function hasPython() {
  const exe = process.platform === 'win32' ? 'py' : 'python3';
  const r = spawnSync(exe, ['--version'], { encoding: 'utf8', shell: process.platform === 'win32' });
  return r.status === 0 ? (r.stdout || r.stderr).trim() : null;
}

async function install(pkgRoot, version, want) {
  banner(version);
  const p = await choose(want);
  if (!p) { if (want) console.log(`  Unknown platform "${want}". Use windows or mac.`); else console.log('  Nothing installed.'); return 1; }
  const url = assetUrl(version, p);
  console.log('');

  // Installing for the other system: it has to be done on that machine. Say so, and say whether its build is out.
  if (p.os !== process.platform) {
    const there = fs.existsSync(path.join(pkgRoot, 'bin', 'native', p.key, p.exe)) || await exists(url);
    console.log(`  ${s.bold(p.label)} runs on ${p.os === 'darwin' ? 'a Mac' : 'Windows'}, so install it there:`);
    console.log(`    ${s.cmd('npx morceau')}  ${s.dim('then pick ' + (p.os === 'darwin' ? 'Install for Mac' : 'Install for Windows'))}`);
    console.log(there ? `  ${s.tick()} The ${p.label} build for v${version} is available.` : `  ${s.warn('!')} The ${p.label} build for v${version} is not out yet. It will be at ${s.dim(url)}`);
    console.log('');
    return 0;
  }

  console.log(`  Installing ${s.bold(p.label)} into ${s.dim(places.runtime)}`);
  if (fs.existsSync(places.bin)) {
    spawnSync(places.bin, ['stop'], { encoding: 'utf8', timeout: 15000 });   // a running host locks its exe on Windows
    if (!places.installed()) {   // an engine this installer did not put there (a dev build): keep it
      fs.copyFileSync(places.bin, places.bin + '.bak');
      step(`kept your existing engine as ${s.dim(path.basename(places.bin) + '.bak')}`);
    }
  }
  // Where the engine comes from, first match wins: a test/mirror folder, the build shipped inside this npm package, the GitHub release.
  const local = process.env.MORC_RELEASE_DIR && path.join(process.env.MORC_RELEASE_DIR, p.asset);
  const bundled = path.join(pkgRoot, 'bin', 'native', p.key, p.exe);
  const from = local ? 'copied from ' + process.env.MORC_RELEASE_DIR : fs.existsSync(bundled) ? 'included in the package' : 'downloaded from GitHub';
  try {
    fs.mkdirSync(path.dirname(places.bin), { recursive: true });
    if (local) fs.copyFileSync(local, places.bin);
    else if (fs.existsSync(bundled)) fs.copyFileSync(bundled, places.bin);
    else await download(url, places.bin, `engine from GitHub (v${version})`);
  } catch (e) {
    return fail(`Could not get the ${p.label} engine.`, `${e.message}. Check your connection, or build from source: see ${`https://github.com/${REPO}#build`}`);
  }
  if (p.os !== 'win32') fs.chmodSync(places.bin, 0o755);
  step(`engine ${from} ${s.dim('(' + p.key + ')')}`);

  const app = copyApp(pkgRoot);
  step(`app files in place ${s.dim('(' + app + ')')}`);
  const data = dataFolder();
  step(`your data folder ${s.dim('(' + data + ')')}`);

  const py = hasPython();
  if (py) step(`${py} found ${s.dim('(the plugins use it)')}`);
  else console.log(`  ${s.warn('!')} Python 3 not found. Cards work without it, but the plugins (init, compose, free, ...) need it: https://www.python.org/downloads/`);

  if (py) {
    console.log(`  ${s.dim('starting Morceau for the first time (can take a few seconds) ...')}`);
    const r = spawnSync(places.bin, ['init'], { encoding: 'utf8', env: { ...process.env, MORC_BIN: places.bin } });
    if (r.status !== 0) return fail('Morceau did not start.', (r.stderr || r.stdout || '').trim().replace(/^morc: /, '').split('\n').slice(-3).join(' '));
    step('started, and the starter cards are saved');
  }

  fs.writeFileSync(places.installedFile, JSON.stringify({ version, platform: p.key, at: new Date().toISOString() }, null, 2));
  console.log('');
  console.log(`  ${s.ok(s.bold('Morceau is installed.'))}`);
  console.log('');
  console.log(`  ${s.bold('Try it')}`);
  console.log(`    ${s.cmd('npx morceau use ask-permission')}   ${s.dim('show a sample card')}`);
  console.log(`    ${s.cmd('npx morceau')}                      ${s.dim('what you can do next')}`);
  console.log(`  ${s.bold('Use it as')} ${s.cmd('morc')} ${s.dim('anywhere:')} ${s.cmd('npm i -g morceau')}`);
  console.log('');
  return 0;
}

module.exports = { install, banner, PLATFORMS };
