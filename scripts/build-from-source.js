// Runs as the "prepare" lifecycle: on `npm install` from a git clone (npm i -g github:Paper-Bag-dev/morceau-interfaces) and before `npm publish`.
// Registry installs ship a prebuilt bin/morc.exe in the tarball and never run this. If the exe already exists there is nothing to do.
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const root = path.resolve(__dirname, '..');
const exe = path.join(root, 'bin', 'morc.exe');

if (process.platform !== 'win32') { console.log('morceau: Windows only, skipping the build'); process.exit(0); }
if (fs.existsSync(exe)) process.exit(0);

console.log('morceau: building morc.exe from source (needs the Rust toolchain, https://rustup.rs) ...');
const r = spawnSync('cargo', ['build', '--release'], { cwd: root, stdio: 'inherit', shell: true });
const built = path.join(root, 'target', 'release', 'morc.exe');
if (r.status !== 0 || !fs.existsSync(built)) {
  console.error('\nmorceau: the build failed. Install Rust (https://rustup.rs) and the MSVC build tools, then run:  node scripts/build-from-source.js');
  console.error('Or install the prebuilt package from the registry instead:  npm i -g morceau');
  process.exit(1);
}
fs.mkdirSync(path.dirname(exe), { recursive: true });
fs.copyFileSync(built, exe);
console.log('morceau: built bin/morc.exe');
