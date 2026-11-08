// donut: share of a whole (where did the time go). Slices sweep in clockwise; the hovered slice pushes outward.
import { palette, easeOutCubic, clamp01, label, fmt, ink } from './util.js';

function geo(s) {
  const cx = s.W * 0.34, cy = s.H / 2, R = Math.min(s.W * 0.34, s.H / 2) - 6;
  return { cx, cy, R, r: R * 0.62 };
}

// labels longer than the legend column are cut with an ellipsis, never clipped by the canvas edge\nconst cut = (t, n) => (t.length > n ? t.slice(0, n - 1) + '\u2026' : t);\n\nexport function draw(g, s) {
  const v = s.values.map((x) => Math.max(0, x)), total = v.reduce((a, b) => a + b, 0) || 1, n = v.length;
  const { cx, cy, R, r } = geo(s);
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
  label(g, hv >= 0 ? cut(String(s.labels[hv] || ''), 14) : 'total', cx, cy + 13, { size: 10.5, color: ink(.6) });
  // legend on the right
  const maxc = Math.max(6, Math.floor((s.W - s.W * 0.7 - 20) / 5.6));   // characters that fit beside the swatch\n  v.slice(0, 7).forEach((val, i) => {
    const y = 14 + i * 17;
    g.fillStyle = palette(i, n, 1, 110); g.beginPath(); g.roundRect(s.W * 0.7, y - 5, 10, 10, 3); g.fill();
    label(g, cut(`${s.labels[i] || i + 1}`, maxc), s.W * 0.7 + 16, y, { align: 'left', size: 10.5, color: i === s.hover ? ink(1) : ink(.7), weight: i === s.hover ? 700 : 500 });
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

