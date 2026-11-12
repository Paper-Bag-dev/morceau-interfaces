// Runs as the "prepare" lifecycle: on `npm install` from a git clone (npm i -g github:Paper-Bag-dev/morceau-interfaces) and before `npm publish`.
// Builds this machine's engine into bin/native/<platform>/ (scripts/npm/platforms.js). Registry installs ship the builds in the tarball
// or fetch them from the GitHub release, so they never run this. Nothing to do when this machine's build is already there.
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { detect } = require('./platforms.js');

const root = path.resolve(__dirname, '..', '..');
const p = detect();
if (!p) { console.log(`morceau: no build for ${process.platform}/${process.arch}, skipping`); process.exit(0); }
const dest = path.join(root, 'bin', 'native', p.key, p.exe);
if (fs.existsSync(dest)) process.exit(0);

console.log(`morceau: building the ${p.label} engine from source (needs the Rust toolchain, https://rustup.rs) ...`);
const r = spawnSync('cargo', ['build', '--release'], { cwd: root, stdio: 'inherit', shell: process.platform === 'win32' });
const target = process.env.CARGO_TARGET_DIR || path.join(root, 'target');
const built = path.join(target, 'release', p.exe);
if (r.status !== 0 || !fs.existsSync(built)) {
  console.error(`\nmorceau: the build failed. Install Rust (https://rustup.rs)${p.os === 'win32' ? ' and the MSVC build tools' : ' and the Xcode command line tools'}, then run:  node scripts/npm/build-from-source.js`);
  console.error('Or install the prebuilt package from the registry instead:  npm i -g morceau');
  process.exit(1);
}
fs.mkdirSync(path.dirname(dest), { recursive: true });
fs.copyFileSync(built, dest);
if (p.os !== 'win32') fs.chmodSync(dest, 0o755);
console.log(`morceau: built bin/native/${p.key}/${p.exe}`);
