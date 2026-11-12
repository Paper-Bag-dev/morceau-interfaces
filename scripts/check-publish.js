// Runs before `npm publish`: refuse to publish anything that should not leave this machine.
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const root = path.resolve(__dirname, '..');
const problems = [];

// Every platform the package claims (package.json "os") must carry its engine, or `npx morceau` there fails at once.
// Windows: scripts/release-build.ps1 on Windows. macOS: scripts/release-build-mac.sh on a Mac (arm64, plus x64 when that target is installed).
const need = { 'morc.exe': 'Windows x64 (scripts/release-build.ps1)', 'morc-darwin-arm64': 'macOS arm64 (scripts/release-build-mac.sh)' };
for (const [f, how] of Object.entries(need)) if (!fs.existsSync(path.join(root, 'bin', f))) problems.push(`bin/${f} is missing: build ${how}`);
if (!fs.existsSync(path.join(root, 'bin', 'morc-darwin-x64'))) console.log('note: no bin/morc-darwin-x64: Intel Macs get a clear "missing" message (rustup target add x86_64-apple-darwin, then rebuild)');
for (const f of ['.env', 'prefs', 'triggers', 'actions.json']) if (fs.existsSync(path.join(root, f))) problems.push(`${f} exists in the package folder: personal data must not be published (use the personal branch or a separate data folder)`);

const pack = spawnSync('npm', ['pack', '--dry-run', '--json'], { cwd: root, shell: true, encoding: 'utf8' });
try {
  const files = JSON.parse(pack.stdout)[0].files.map((x) => x.path);
  const bad = files.filter((p) => /(^|\/)\.env$|^prefs\/|^triggers\/|^actions\.json$|models\/assets\/|\.log$|(^|\/)(\.?token|secrets?|credentials?)(\.|$)/i.test(p));   // a file NAMED token/secret/credentials, not design "tokens" (ui/tokens/, tokens-cards.json)
  if (bad.length) problems.push('these files would be published: ' + bad.join(', '));
  console.log(`package check: ${files.length} files`);
} catch (e) {
  problems.push('could not read the pack list: ' + (pack.stderr || e.message));
}

if (problems.length) { console.error('\nNOT publishing:\n - ' + problems.join('\n - ')); process.exit(1); }
console.log('publish check passed');
