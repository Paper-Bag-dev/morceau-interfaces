// Runs before `npm publish`: refuse to publish anything that should not leave this machine, or a version whose engine is not on GitHub.
// The package carries NO engine (bin/native/ is excluded in package.json "files"): the installer (bin/launcher/install.js) downloads each
// platform's engine from the GitHub release v<version>. So the Windows engine must already be on that release; missing Mac engines are
// reported (the installer tells Mac users the build is not out yet) but do not block.
const fs = require('fs');
const path = require('path');
const https = require('https');
const { spawnSync } = require('child_process');
const { PLATFORMS } = require('./platforms.js');

const root = path.resolve(__dirname, '..', '..');
const version = require(path.join(root, 'package.json')).version;
const problems = [];

function exists(url, hops = 0) {
  return new Promise((ok) => https.get(url, { headers: { 'User-Agent': 'morceau-publish-check' } }, (r) => {
    r.resume();
    if (r.statusCode >= 300 && r.statusCode < 400 && r.headers.location && hops < 6) return exists(r.headers.location, hops + 1).then(ok);
    ok(r.statusCode === 200);
  }).on('error', () => ok(false)));
}

async function releaseCheck() {
  for (const p of PLATFORMS) {
    const url = `https://github.com/Paper-Bag-dev/morceau-interfaces/releases/download/v${version}/${p.asset}`;
    const there = await exists(url);
    console.log(`${there ? 'on GitHub' : 'MISSING  '}  ${p.key}  ${url}`);
    if (!there && p.os === 'win32') problems.push(`the ${p.label} engine is not on the GitHub release v${version}: upload ${p.asset} first`);
  }
}
for (const f of ['.env', 'prefs', 'triggers', 'actions.json']) if (fs.existsSync(path.join(root, f))) problems.push(`${f} exists in the package folder: personal data must not be published (use the personal branch or a separate data folder)`);

const pack = spawnSync('npm', ['pack', '--dry-run', '--json'], { cwd: root, shell: process.platform === 'win32', encoding: 'utf8' });
try {
  const files = JSON.parse(pack.stdout)[0].files.map((x) => x.path);
  const bad = files.filter((p) => /(^|\/)\.env$|^prefs\/|^triggers\/|^actions\.json$|models\/assets\/|\.log$|^bin\/native\/|(^|\/)(\.?token|secrets?|credentials?)(\.|$)/i.test(p));   // a file NAMED token/secret/credentials, not design "tokens" (ui/tokens/, tokens-cards.json); engines come from GitHub
  if (bad.length) problems.push('these files would be published: ' + bad.join(', '));
  console.log(`package check: ${files.length} files`);
} catch (e) {
  problems.push('could not read the pack list: ' + (pack.stderr || e.message));
}

releaseCheck().then(() => {
  if (problems.length) { console.error('\nNOT publishing:\n - ' + problems.join('\n - ')); process.exit(1); }
  console.log('publish check passed');
});
