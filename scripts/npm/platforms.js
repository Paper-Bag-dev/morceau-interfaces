// The builds Morceau ships, one entry per platform. Shared by the npm launcher (bin/morc.js), the from-source build and the publish
// check, so the names, folders and release asset names exist in one place.
//   key      what `--platform` / MORC_PLATFORM accepts and the folder under bin/native/
//   os, cpu  the Node.js process.platform / process.arch it runs on
//   exe      the binary's name inside bin/native/<key>/
//   asset    the file name on the GitHub release (built by .github/workflows/release.yml)
//   target   the Rust target triple CI builds it for
const PLATFORMS = [
  { key: 'windows-x64', label: 'Windows (x64)', os: 'win32', cpu: 'x64', exe: 'morc.exe', asset: 'morc-windows-x64.exe', target: 'x86_64-pc-windows-msvc' },
  { key: 'macos-arm64', label: 'macOS (Apple silicon)', os: 'darwin', cpu: 'arm64', exe: 'morc', asset: 'morc-macos-arm64', target: 'aarch64-apple-darwin' },
  { key: 'macos-x64', label: 'macOS (Intel)', os: 'darwin', cpu: 'x64', exe: 'morc', asset: 'morc-macos-x64', target: 'x86_64-apple-darwin' },
];

const ALIASES = { windows: 'windows-x64', win: 'windows-x64', win32: 'windows-x64', mac: null, macos: null, darwin: null };

/** The build for this machine, or null when Morceau has none for it. */
function detect() {
  return PLATFORMS.find((p) => p.os === process.platform && p.cpu === process.arch) || null;
}

/** A user's choice ("mac", "windows", "macos-x64", ...) to a platform entry; "mac" picks this Mac's own architecture. */
function resolve(name) {
  if (!name) return detect();
  const n = String(name).toLowerCase();
  if (n in ALIASES) {
    if (ALIASES[n]) return PLATFORMS.find((p) => p.key === ALIASES[n]);
    return PLATFORMS.find((p) => p.os === 'darwin' && p.cpu === (process.arch === 'x64' ? 'x64' : 'arm64'));
  }
  return PLATFORMS.find((p) => p.key === n) || null;
}

module.exports = { PLATFORMS, detect, resolve };
