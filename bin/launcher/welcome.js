// What `npx morceau` (no arguments) shows once Morceau is installed: the version, whether it is running, and the few commands
// a person needs first, each with one plain line. `help` still prints the engine's full command list.
const net = require('net');
const places = require('./places.js');
const { banner } = require('./install.js');
const s = require('./style.js');

function hostUp() {
  return new Promise((ok) => {
    const c = net.connect({ host: '127.0.0.1', port: 47831 });
    const end = (v) => { c.destroy(); ok(v); };
    c.setTimeout(250, () => end(false));
    c.on('connect', () => end(true));
    c.on('error', () => end(false));
  });
}

async function welcome(version, prefix) {
  banner(version);
  const inst = places.installed();
  const up = await hostUp();
  console.log(`  ${s.tick()} installed${inst ? s.dim(` (v${inst.version}, ${inst.platform})`) : ''}   ${up ? s.ok('●') + ' running' : s.dim('○ not running, starts on the first command')}`);
  if (inst && inst.version !== version) console.log(`  ${s.warn('!')} v${version} is out: ${s.cmd(prefix + ' install')} updates the engine and app files (your data stays)`);
  console.log('');
  const rows = [
    ['use ask-permission', 'show a sample card'],
    ['list', 'the cards you have saved'],
    ['status', "what's on screen now"],
    ['dismiss-all', 'clear the screen'],
    ['commands', 'the tools (init, compose, free, ...)'],
    ['help', 'every command, in detail'],
  ];
  const w = Math.max(...rows.map((r) => r[0].length));
  console.log(`  ${s.bold('Commands')}`);
  for (const [c, d] of rows) console.log(`    ${s.cmd(prefix + ' ' + c.padEnd(w))}   ${s.dim(d)}`);
  console.log('');
  console.log(`  ${s.dim('Reinstall or update:')} ${s.cmd(prefix + ' install')}   ${s.dim('Docs:')} https://github.com/Paper-Bag-dev/morceau-interfaces`);
  console.log('');
}

module.exports = { welcome };
