// radar: a profile across 3+ axes. Five axes is the classic pentagon; any n-gon works. Great for "how balanced is my week".
import { base, easeOutBack, clamp01, label, fmt, ink } from './util.js';

function geo(s) {
  const n = s.values.length, cx = s.W / 2, cy = s.H / 2 + 4, R = Math.min(s.W, s.H) / 2 - 26;
  const ang = (i) => -Math.PI / 2 + (i * 2 * Math.PI) / n;
  return { n, cx, cy, R, ang };
}

export function draw(g, s) {
  const { values: v, t, hover, c } = s;
  const { n, cx, cy, R, ang } = geo(s);
  if (n < 3) return;
  const max = c.max || Math.max(1, ...v, ...s.target);
  const P = (i, r) => [cx + Math.cos(ang(i)) * r, cy + Math.sin(ang(i)) * r];

  // web: concentric n-gons + spokes
  g.strokeStyle = ink(.1); g.lineWidth = 1;
  for (let ring = 1; ring <= 4; ring++) {
    g.beginPath();
    for (let i = 0; i < n; i++) { const [x, y] = P(i, (R * ring) / 4); i ? g.lineTo(x, y) : g.moveTo(x, y); }
    g.closePath(); g.stroke();
  }
  for (let i = 0; i < n; i++) { const [x, y] = P(i, R); g.beginPath(); g.moveTo(cx, cy); g.lineTo(x, y); g.stroke(); }

  // the shape blooms out from the center, rotating a few degrees as it settles
  const grow = easeOutBack(clamp01(t)), spin = (1 - clamp01(t)) * -0.5;
  const Q = (i, r) => [cx + Math.cos(ang(i) + spin) * r, cy + Math.sin(ang(i) + spin) * r];
  g.beginPath();
  v.forEach((val, i) => { const [x, y] = Q(i, (Math.min(val, max) / max) * R * grow); i ? g.lineTo(x, y) : g.moveTo(x, y); });
  g.closePath();
  g.fillStyle = base(0.28); g.fill();
  g.strokeStyle = base(1); g.lineWidth = 2.2; g.shadowColor = base(0.6); g.shadowBlur = 12; g.stroke(); g.shadowBlur = 0;
  v.forEach((val, i) => { const [x, y] = Q(i, (Math.min(val, max) / max) * R * grow); g.beginPath(); g.arc(x, y, i === hover ? 5 : 3.2, 0, 7); g.fillStyle = i === hover ? ink(1) : base(1); g.fill(); });

  s.labels.slice(0, n).forEach((l, i) => { const [x, y] = P(i, R + 13); label(g, i === hover ? `${l} ${fmt(v[i], s.unit)}` : l, x, y, { size: 10.5, color: i === hover ? ink(1) : ink(.65), weight: i === hover ? 700 : 500 }); });
}

export function hit(s, x, y) {
  const { n, cx, cy, ang, R } = geo(s);
  let best = -1, bd = 1e9;
  for (let i = 0; i < n; i++) { const d = Math.hypot(cx + Math.cos(ang(i)) * R - x, cy + Math.sin(ang(i)) * R - y); if (d < bd) { bd = d; best = i; } }
  return bd < R ? best : -1;
}

