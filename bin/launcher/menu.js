// A tiny pick-one menu for the terminal: arrow keys (or j/k) move, Enter picks, a number picks directly, Ctrl+C or q leaves.
// Each option is {label, hint}. Returns the chosen index, or null when the user leaves. Falls back to a typed number off a TTY.
const readline = require('readline');
const s = require('./style.js');

function draw(options, i, first) {
  if (!first) process.stdout.write(`\x1b[${options.length}A`);   // back to the top of the list
  options.forEach((o, k) => {
    const sel = k === i;
    const line = `${sel ? s.accent('❯') : ' '} ${sel ? s.bold(o.label) : o.label}${o.hint ? '  ' + s.dim(o.hint) : ''}`;
    process.stdout.write(`\x1b[2K  ${line}\n`);
  });
}

function pick(options, start = 0) {
  if (!process.stdin.isTTY || !process.stdout.isTTY) {
    return new Promise((ok) => {
      options.forEach((o, k) => console.log(`  ${k + 1}) ${o.label}`));
      const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
      rl.question(`Choose 1-${options.length}: `, (a) => { rl.close(); const n = parseInt(a, 10); ok(n >= 1 && n <= options.length ? n - 1 : null); });
    });
  }
  return new Promise((ok) => {
    let i = start;
    readline.emitKeypressEvents(process.stdin);
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdout.write('\x1b[?25l');   // hide the cursor while choosing
    draw(options, i, true);
    const done = (v) => {
      process.stdin.removeListener('keypress', onKey);
      process.stdin.setRawMode(false);
      process.stdin.pause();
      process.stdout.write('\x1b[?25h');
      ok(v);
    };
    const onKey = (str, key = {}) => {
      if ((key.ctrl && key.name === 'c') || key.name === 'escape' || str === 'q') return done(null);
      if (key.name === 'up' || str === 'k') i = (i + options.length - 1) % options.length;
      else if (key.name === 'down' || str === 'j' || key.name === 'tab') i = (i + 1) % options.length;
      else if (key.name === 'return') return done(i);
      else if (/^[1-9]$/.test(str || '') && +str <= options.length) { i = +str - 1; draw(options, i, false); return done(i); }
      draw(options, i, false);
    };
    process.stdin.on('keypress', onKey);
  });
}

module.exports = { pick };
