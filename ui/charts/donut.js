// donut: share of a whole (where did the time go). Slices sweep in clockwise; the hovered slice pushes outward.
// Layout is padding + flow, never fixed fractions: [pad][ring][gap][swatch + label column][pad]. width() tells the chart how wide it needs to be.
import { palette, easeOutCubic, clamp01, label, fmt, ink, measure } from './util.js';

const PAD = 14, GAP = 22, ROW = 17, SW = 10, SWGAP = 6, MAXROWS = 7;

function geo(s) {
  const R = Math.max(20, s.H / 2 - PAD);
  return { cx: PAD + R, cy: s.H / 2, R, r: R * 0.62, lx: PAD + R * 2 + GAP };
}

// the width this chart needs so every legend label fits inside it, with padding all round
export function width(s) {
  const { lx } = geo(s);
  const longest = Math.max(0, ...(s.labels || []).slice(0, MAXROWS).map((t) => measure(t, 10.5, 700)));
  return Math.ceil(lx + SW + SWGAP + longest + PAD);
}

export function draw(g, s) {
  const v = s.values.map((x) => Math.max(0, x)), total = v.reduce((a, b) => a + b, 0) || 1, n = v.length;
  const { cx, cy, R, r, lx } = geo(s);
  const sweep = easeOutCubic(clamp01(s.t)) * Math.PI * 2;
  let a = -Math.PI / 2;
  v.forEach((val, i) => {
    const span = (val / total) * Math.PI * 2, end = Math.min(a + span, -Math.PI / 2 + sweep);
    if (end > a) {
      const mid = (a + end) / 2, push = i === s.hover ? 6 : 0;
      const ox = Math.cos(mid) * push, oy = Math.sin(mid) * push;
      g.beginPath(); g.arc(cx + ox, cy + oy, R, a, end); g.arc(cx + ox, cy + oy, r, end, a, true); g.closePath();
      g.fillStyle = palette(i, n, i === s.hover ? 1 : 0.85, 110); g.fill();
      g.strokeStyle = 'rgba(0,0,0,.35)'; g.lineWidth = 1.5; g.stroke();
    }
    a += span;
  });
  const hv = s.hover >= 0 ? s.hover : -1;
  label(g, hv >= 0 ? fmt(v[hv], s.unit) : fmt(total, s.unit), cx, cy - 4, { size: 19, weight: 600, color: ink(1), display: true });
  const inner = r * 2 - 8, name = hv >= 0 ? String(s.labels[hv] || '') : 'total';
  label(g, measure(name, 10.5) > inner ? name.slice(0, Math.max(3, Math.floor(name.length * inner / measure(name, 10.5)) - 1)) + '\u2026' : name, cx, cy + 13, { size: 10.5, color: ink(.6) });
  // legend: a column vertically centred beside the ring
  const rows = Math.min(n, MAXROWS), top = cy - (rows * ROW) / 2 + ROW / 2;
  v.slice(0, MAXROWS).forEach((val, i) => {
    const y = top + i * ROW;
    g.fillStyle = palette(i, n, 1, 110); g.beginPath(); g.roundRect(lx, y - SW / 2, SW, SW, 3); g.fill();
    label(g, `${s.labels[i] || i + 1}`, lx + SW + SWGAP, y, { align: 'left', size: 10.5, color: i === s.hover ? ink(1) : ink(.7), weight: i === s.hover ? 700 : 500 });
  });
}

export function hit(s, x, y) {
  const { cx, cy, R, r } = geo(s);
  const d = Math.hypot(x - cx, y - cy);
  if (d < r * 0.9 || d > R + 8) return -1;
  let ang = Math.atan2(y - cy, x - cx) + Math.PI / 2;
  if (ang < 0) ang += Math.PI * 2;
  const v = s.values, total = v.reduce((a, b) => a + Math.max(0, b), 0) || 1;
  let acc = 0;
  for (let i = 0; i < v.length; i++) { acc += (Math.max(0, v[i]) / total) * Math.PI * 2; if (ang <= acc) return i; }
  return -1;
}
