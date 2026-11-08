// treemap: area = share of the total. A pie chart that can hold long names. Boxes pop in largest-first.
import { palette, easeOutBack, stagger, label, fmt, ink } from './util.js';

/** Balanced binary split: halve the items by sum, cut the rectangle along its longer side. */
function layout(items, x, y, w, h, out) {
  if (items.length === 1) return out.push({ ...items[0], x, y, w, h });
  const total = items.reduce((a, b) => a + b.v, 0) || 1;
  let acc = 0, k = 0;
  while (k < items.length - 1 && acc + items[k].v / 2 < total / 2) acc += items[k++].v;
  k = Math.max(1, k);
  const a = items.slice(0, k), b = items.slice(k), fa = a.reduce((p, q) => p + q.v, 0) / total;
  if (w >= h) { layout(a, x, y, w * fa, h, out); layout(b, x + w * fa, y, w * (1 - fa), h, out); }
  else { layout(a, x, y, w, h * fa, out); layout(b, x, y + h * fa, w, h * (1 - fa), out); }
}

function cells(s) {
  const items = s.values.map((v, i) => ({ v: Math.max(0.0001, v), i })).sort((a, b) => b.v - a.v);
  const out = [];
  if (items.length) layout(items, 6, 6, s.W - 12, s.H - 12, out);
  return out;
}

export function draw(g, s) {
  const cs = cells(s);
  cs.forEach((c, rank) => {
    const p = Math.max(0.001, easeOutBack(stagger(s.t, rank, cs.length, 0.6)));
    const cx = c.x + c.w / 2, cy = c.y + c.h / 2, hot = c.i === s.hover;
    g.save(); g.translate(cx, cy); g.scale(p * (hot ? 1.03 : 1), p * (hot ? 1.03 : 1));
    g.fillStyle = palette(c.i, s.values.length, hot ? 1 : 0.78, 120);
    g.beginPath(); g.roundRect(-c.w / 2 + 1.5, -c.h / 2 + 1.5, c.w - 3, c.h - 3, 7); g.fill();
    if (c.w > 52 && c.h > 30) {
      label(g, s.labels[c.i] || `#${c.i + 1}`, 0, -6, { size: 11, weight: 700, color: 'rgba(0,0,0,.78)' });
      label(g, fmt(s.values[c.i], s.unit), 0, 8, { size: 10, color: 'rgba(0,0,0,.6)' });
    }
    g.restore();
  });
}

export function hit(s, x, y) {
  const c = cells(s).find((c) => x >= c.x && x <= c.x + c.w && y >= c.y && y <= c.y + c.h);
  return c ? c.i : -1;
}

