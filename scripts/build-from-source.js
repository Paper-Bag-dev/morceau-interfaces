// Runs as the "prepare" lifecycle: on `npm install` from a git clone (npm i -g github:Paper-Bag-dev/morceau-interfaces) and before `npm publish`.
// Registry installs ship a prebuilt bin/morc.exe in the tarball and never run this. If the exe already exists there is nothing to do.
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const root = path.resolve(__dirname, '..');
// the engine for THIS platform, named the way bin/morc.js looks for it
const name = process.platform === 'win32' ? 'morc.exe' : process.platform === 'darwin' ? `morc-darwin-${process.arch}` : null;
if (!name) { console.log('morceau: Windows and macOS only, skipping the build'); process.exit(0); }
const exe = path.join(root, 'bin', name);
if (fs.existsSync(exe)) process.exit(0);

console.log(`morceau: building ${name} from source (needs the Rust toolchain, https://rustup.rs) ...`);
const r = spawnSync('cargo', ['build', '--release'], { cwd: root, stdio: 'inherit', shell: process.platform === 'win32' });
const built = path.join(root, 'target', 'release', process.platform === 'win32' ? 'morc.exe' : 'morc');
if (r.status !== 0 || !fs.existsSync(built)) {
  console.error('\nmorceau: the build failed. Install Rust (https://rustup.rs) and the platform build tools (Windows: MSVC; macOS: xcode-select --install), then run:  node scripts/build-from-source.js');
  console.error('Or install the prebuilt package from the registry instead:  npm i -g morceau');
  process.exit(1);
}
fs.mkdirSync(path.dirname(exe), { recursive: true });
fs.copyFileSync(built, exe);
if (process.platform !== 'win32') fs.chmodSync(exe, 0o755);
console.log(`morceau: built bin/${name}`);
