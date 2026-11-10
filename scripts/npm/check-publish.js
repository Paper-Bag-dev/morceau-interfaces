// Runs before `npm publish`: refuse to publish anything that should not leave this machine, or a package with no engine in it.
// The package carries every build it has in bin/native/<platform>/ (scripts/npm/platforms.js); a build that is missing is fetched by the
// launcher from the GitHub release of the same version, so at least one build must be present and the rest must be on that release.
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { PLATFORMS } = require('./platforms.js');

const root = path.resolve(__dirname, '..', '..');
const problems = [];

const present = PLATFORMS.filter((p) => fs.existsSync(path.join(root, 'bin', 'native', p.key, p.exe)));
if (!present.length) problems.push('no engine in bin/native/: build one first (node scripts/npm/build-from-source.js) or let CI assemble them (.github/workflows/release.yml)');
else console.log(`builds in the package: ${present.map((p) => p.key).join(', ')}` + (present.length < PLATFORMS.length ? ' (the others come from the GitHub release)' : ''));
if (fs.existsSync(path.join(root, 'bin', 'native', 'selected.txt'))) problems.push('bin/native/selected.txt is a local choice: delete it before publishing');
for (const f of ['.env', 'prefs', 'triggers', 'actions.json']) if (fs.existsSync(path.join(root, f))) problems.push(`${f} exists in the package folder: personal data must not be published (use the personal branch or a separate data folder)`);

const pack = spawnSync('npm', ['pack', '--dry-run', '--json'], { cwd: root, shell: process.platform === 'win32', encoding: 'utf8' });
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
