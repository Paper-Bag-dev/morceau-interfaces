// Runs before `npm publish`: refuse to publish anything that should not leave this machine.
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const root = path.resolve(__dirname, '..');
const problems = [];

if (!fs.existsSync(path.join(root, 'bin', 'morc.exe'))) problems.push('bin/morc.exe is missing: build it first (cargo build --release; node scripts/build-from-source.js)');
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
