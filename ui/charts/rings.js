// rings: concentric progress rings, one per goal (like fitness rings). Full circle = target hit; past it = overshoot,
// drawn as a second lap in a brighter shade. `targets` gives each ring's goal (else `max` for all).
import { palette, easeOutBack, clamp01, stagger, label, fmt, ink } from './util.js';

function geo(s) {
  const n = s.values.length, R = Math.min(s.W * 0.36, s.H / 2) - 4, gap = 4;
  const th = Math.min(15, (R * 0.8) / n - gap);
  return { n, cx: s.W * 0.36, cy: s.H / 2, R, th, gap };
}

export function draw(g, s) {
  const { n, cx, cy, R, th, gap } = geo(s);
  g.lineCap = 'round';
  s.values.forEach((val, i) => {
    const goal = (s.c.targets && s.c.targets[i]) || s.c.max || 100, r = R - i * (th + gap) - th / 2;
    const ratio = (val / goal) * easeOutBack(stagger(s.t, i, n, 0.5));
    g.lineWidth = th; g.strokeStyle = ink(.08);
    g.beginPath(); g.arc(cx, cy, r, 0, Math.PI * 2); g.stroke();
    const col = palette(i, n, i === s.hover ? 1 : 0.92, 100);
    g.strokeStyle = col; g.shadowColor = col; g.shadowBlur = 10;
    g.beginPath(); g.arc(cx, cy, r, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * clamp01(ratio)); g.stroke();
    if (ratio > 1) { g.strokeStyle = ink(.55); g.beginPath(); g.arc(cx, cy, r, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * clamp01(ratio - 1)); g.stroke(); }
    g.shadowBlur = 0;
  });
  s.values.slice(0, 6).forEach((val, i) => {
    const goal = (s.c.targets && s.c.targets[i]) || s.c.max || 100, y = 16 + i * 22, x = s.W * 0.72;
    g.fillStyle = palette(i, n, 1, 100); g.beginPath(); g.arc(x, y, 5, 0, 7); g.fill();
    label(g, s.labels[i] || `#${i + 1}`, x + 12, y - 5, { align: 'left', size: 10.5, color: i === s.hover ? ink(1) : ink(.75), weight: 600 });
    label(g, `${fmt(val, s.unit)} / ${goal}`, x + 12, y + 7, { align: 'left', size: 9.5, color: ink(.5) });
  });
}

export function hit(s, x, y) {
  const { n, cx, cy, R, th, gap } = geo(s), d = Math.hypot(x - cx, y - cy);
  const i = Math.floor((R - d) / (th + gap));
  return i >= 0 && i < n ? i : -1;
}

