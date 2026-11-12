// Colors for the launcher's screens. chalk when it is installed (it is a dependency of the npm package), plain ANSI otherwise
// (a git checkout without `npm install`), and no color at all when the output is not a terminal or NO_COLOR is set.
let chalk = null;
try { chalk = require('chalk'); } catch { /* fallback below */ }

const on = process.stdout.isTTY && !process.env.NO_COLOR;
const ansi = (open, close) => (s) => (on ? `\x1b[${open}m${s}\x1b[${close}m` : String(s));
const fb = {
  bold: ansi(1, 22), dim: ansi(2, 22), red: ansi(31, 39), green: ansi(32, 39), yellow: ansi(33, 39), cyan: ansi(36, 39), magenta: ansi(35, 39),
};
const c = chalk && on ? chalk : fb;

module.exports = {
  bold: (s) => c.bold(s),
  dim: (s) => c.dim(s),
  accent: (s) => (chalk && on ? chalk.hex('#FF7A59')(s) : fb.magenta(s)),   // morc's coral
  ok: (s) => c.green(s),
  warn: (s) => c.yellow(s),
  bad: (s) => c.red(s),
  cmd: (s) => c.cyan(s),
  tick: () => c.green('✓'),
  cross: () => c.red('✗'),
  color: on,
};
