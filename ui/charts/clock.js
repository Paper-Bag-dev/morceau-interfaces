// clock: 24 wedges around a dial, one per hour; reach = how much happened in that hour.
// Made for "when do I leak": your 8 PM to 12:30 AM peak shows as a bulge on the lower-left of the dial.
import { base, easeOutBack, stagger, label, fmt, ink } from './util.js';

function geo(s) {
  const cx = s.W / 2, cy = s.H / 2 + 2, R = Math.min(s.W, s.H) / 2 - 18;
  return { cx, cy, R, r: R * 0.28 };
}
const ang = (h) => -Math.PI / 2 + (h / 24) * Math.PI * 2;

export function draw(g, s) {
  const { cx, cy, R, r } = geo(s);
  const v = s.values.slice(0, 24), max = Math.max(1, ...v, ...s.target);
  g.strokeStyle = ink(.08); g.lineWidth = 1;
  for (const k of [0.5, 1]) { g.beginPath(); g.arc(cx, cy, r + (R - r) * k, 0, 7); g.stroke(); }
  const nowH = new Date().getHours();
  v.forEach((val, h) => {
    const p = easeOutBack(stagger(s.t, h, 24, 0.6));
    const len = r + (R - r) * (val / max) * Math.max(0, p);
    const a0 = ang(h) + 0.02, a1 = ang(h + 1) - 0.02;
    g.beginPath(); g.arc(cx, cy, r, a0, a1); g.arc(cx, cy, Math.max(r, len), a1, a0, true); g.closePath();
    g.fillStyle = base(h === s.hover ? 1 : 0.35 + 0.6 * (val / max)); g.fill();
    if (h === nowH) { g.strokeStyle = ink(1); g.lineWidth = 1.5; g.stroke(); }
  });
  [['12a', 0], ['6a', 6], ['12p', 12], ['6p', 18]].forEach(([t, h]) => {
    const a = ang(h), x = cx + Math.cos(a) * (R + 11), y = cy + Math.sin(a) * (R + 11);
    label(g, t, x, y, { size: 10, color: ink(.55) });
  });
  const hv = s.hover;
  label(g, hv >= 0 ? fmt(v[hv], s.unit) : '24h', cx, cy - 3, { size: hv >= 0 ? 16 : 14, weight: 600, color: ink(1), display: true });
  label(g, hv >= 0 ? `${String(hv).padStart(2, '0')}:00` : 'hover a wedge', cx, cy + 12, { size: 9.5, color: ink(.55) });
}

export function hit(s, x, y) {
  const { cx, cy, R, r } = geo(s), d = Math.hypot(x - cx, y - cy);
  if (d < r || d > R + 6) return -1;
  let a = Math.atan2(y - cy, x - cx) + Math.PI / 2;
  if (a < 0) a += Math.PI * 2;
  return Math.min(23, Math.floor((a / (Math.PI * 2)) * 24));
}

