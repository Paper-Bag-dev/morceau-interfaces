// HTTPS download with redirects (GitHub release assets redirect to a CDN) and a one-line progress bar.
const fs = require('fs');
const https = require('https');
const path = require('path');
const s = require('./style.js');

const UA = { 'User-Agent': 'morceau-launcher' };

function get(url, hops = 0) {
  return new Promise((ok, fail) => {
    https.get(url, { headers: UA }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location && hops < 6) { res.resume(); return get(res.headers.location, hops + 1).then(ok, fail); }
      ok(res);
    }).on('error', fail);
  });
}

/** True when the URL exists (follows redirects, reads nothing). */
async function exists(url) {
  try { const r = await get(url); r.resume(); return r.statusCode === 200; } catch { return false; }
}

/** Download to dest (atomic: .part then rename), drawing "label  [#####-----] 42%  1.1 MB". */
async function download(url, dest, label) {
  const res = await get(url);
  if (res.statusCode !== 200) { res.resume(); throw new Error(`HTTP ${res.statusCode} for ${url}`); }
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  const total = +res.headers['content-length'] || 0;
  let got = 0;
  const tmp = dest + '.part';
  const out = fs.createWriteStream(tmp);
  const bar = () => {
    if (!process.stdout.isTTY) return;
    const mb = (got / 1048576).toFixed(1) + ' MB';
    if (!total) return process.stdout.write(`\r\x1b[2K  ${label}  ${s.dim(mb)}`);
    const f = got / total, n = 24, full = Math.round(f * n);
    process.stdout.write(`\r\x1b[2K  ${label}  ${s.accent('█'.repeat(full))}${s.dim('░'.repeat(n - full))} ${Math.round(f * 100)}%  ${s.dim(mb)}`);
  };
  res.on('data', (d) => { got += d.length; bar(); });
  await new Promise((ok, fail) => { res.pipe(out); out.on('finish', ok); out.on('error', fail); res.on('error', fail); });
  await new Promise((ok) => out.close(ok));
  fs.renameSync(tmp, dest);
  if (process.stdout.isTTY) process.stdout.write('\r\x1b[2K');
}

module.exports = { download, exists };
