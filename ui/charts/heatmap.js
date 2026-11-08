// heatmap: a grid of intensities (hour-of-day x day-of-week, or just the last N days). Needs `cols`.
// Cells pop in with a diagonal ripple from the top-left; the hovered cell lifts.
import { base, easeOutBack, stagger, label, fmt, ink } from './util.js';

function geo(s) {
  const cols = s.c.cols || 7, rows = Math.ceil(s.values.length / cols);
  const gx = 4, w = (s.W - 16 - gx * (cols - 1)) / cols, h = Math.min(w, (s.H - 14 - gx * (rows - 1)) / rows);
  return { cols, rows, gx, w, h };
}

export function draw(g, s) {
  const { cols, rows, gx, w, h } = geo(s);
  const max = s.c.max || Math.max(1, ...s.values, ...s.target);
  s.values.forEach((val, i) => {
    const cx = i % cols, cy = Math.floor(i / cols);
    const p = easeOutBack(stagger(s.t, cx + cy, cols + rows, 0.7));
    const x = 8 + cx * (w + gx), y = 8 + cy * (h + gx), k = Math.max(0.001, p);
    const lift = i === s.hover ? 1.18 : 1;
    g.save();
    g.translate(x + w / 2, y + h / 2); g.scale(k * lift, k * lift);
    g.fillStyle = val <= 0 ? ink(.06) : base(0.18 + 0.82 * Math.min(1, val / max));
    g.beginPath(); g.roundRect(-w / 2, -h / 2, w, h, 4); g.fill();
    g.restore();
  });
  if (s.hover >= 0) label(g, `${s.labels[s.hover] ? s.labels[s.hover] + ': ' : ''}${fmt(s.values[s.hover], s.unit)}`, s.W / 2, s.H - 5, { weight: 600 });
}

export function hit(s, x, y) {
  const { cols, rows, gx, w, h } = geo(s);
  const cx = Math.floor((x - 8) / (w + gx)), cy = Math.floor((y - 8) / (h + gx));
  const i = cy * cols + cx;
  return cx >= 0 && cx < cols && cy >= 0 && cy < rows && i < s.values.length ? i : -1;
}

