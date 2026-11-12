// line / area / spark / bar: trends and comparisons over an ordered axis.
import { base, palette, easeOutBack, easeOutCubic, stagger, label, fmt, ink, measure } from './util.js';
import { limit } from '../core/bridge.js';

const PAD = 10;

// A bar chart is exactly as wide as its bars and labels need (a slot per bar wide enough for its label, plus padding), never a default
// width with an empty strip that the window can cut off. Few bars make a narrow chart, many bars a wider one. Other kinds keep the default.
// the key part of a line / area / bar chart: the same chart drawn narrower (limits chart-key-frac of its width); hovering widens it to
// the full width and labels every value (key + detail, docs/theme-rules.md 14). Sparks are already their own key.
export function keyWidth(s) {
  if (s.kind === 'spark') return 0;
  return Math.round((s.c.width || 280) * limit('chart-key-frac', 0.62));
}

export function width(s) {
  if (s.kind !== 'bar' || !(s.labels && s.labels.length)) return 0;
  const longest = Math.max(0, ...s.labels.map((l) => measure(l, 9.5)));
  const n = Math.max(s.labels.length, (s.target || []).length);
  return Math.max(180, Math.ceil(PAD * 2 + n * Math.max(40, longest + 8)));
}

function frame(s) {
  const { W, H, values: v } = s;
  const max = Math.max(1, ...v, ...s.target) * 1.14;
  const n = v.length;
  const bar = s.kind === 'bar';
  const X = (i) => (bar ? PAD + (i + 0.5) * ((W - PAD * 2) / n) : PAD + (i * (W - PAD * 2)) / Math.max(1, n - 1));
  const Y = (val) => H - PAD - 6 - (val / max) * (H - PAD * 2 - 14);
  return { max, n, X, Y, bar };
}

export function draw(g, s) {
  const { W, H, values: v, kind, t, hover } = s;
  if (!v.length) return;
  const { n, X, Y, bar } = frame(s);

  if (kind !== 'spark') {
    g.strokeStyle = ink(.07); g.lineWidth = 1;
    for (let k = 1; k < 4; k++) { const y = PAD + ((H - PAD * 2) * k) / 4; g.beginPath(); g.moveTo(PAD, y); g.lineTo(W - PAD, y); g.stroke(); }
  }

  if (bar) {
    const bw = Math.min(30, ((W - PAD * 2) / n) * 0.62);
    v.forEach((val, i) => {
      const p = easeOutBack(stagger(t, i, n, 0.55));       // each bar pops up in turn, with a little overshoot
      const y0 = Y(0), y = y0 - (y0 - Y(val)) * p;
      g.fillStyle = base(i === hover ? 1 : 0.72);
      g.beginPath(); g.roundRect(X(i) - bw / 2, y, bw, Math.max(0, y0 - y), 5); g.fill();
      if (i === hover) { g.shadowColor = base(0.6); g.shadowBlur = 14; g.fill(); g.shadowBlur = 0; }
    });
  } else {
    // the line draws itself left to right
    const reveal = easeOutCubic(t), upto = (n - 1) * reveal;
    const pts = [];
    for (let i = 0; i <= Math.floor(upto); i++) pts.push([X(i), Y(v[i])]);
    const f = upto - Math.floor(upto);
    if (f > 0 && Math.floor(upto) + 1 < n) { const a = Math.floor(upto); pts.push([X(a) + (X(a + 1) - X(a)) * f, Y(v[a]) + (Y(v[a + 1]) - Y(v[a])) * f]); }
    if (pts.length > 1) {
      g.beginPath(); g.moveTo(...pts[0]);
      for (let i = 1; i < pts.length; i++) { const [px, py] = pts[i - 1], [cx, cy] = pts[i], mx = (px + cx) / 2; g.bezierCurveTo(mx, py, mx, cy, cx, cy); } // smooth
      // the line runs through the theme palette left to right (sparklines stay one color)
      const run = (a) => { const lg = g.createLinearGradient(PAD, 0, W - PAD, 0); for (let k = 0; k <= 4; k++) lg.addColorStop(k / 4, palette(k, 5, a)); return lg; };
      g.strokeStyle = kind === 'spark' ? base(1) : run(1); g.lineWidth = kind === 'spark' ? 1.8 : 2.4; g.lineJoin = 'round'; g.lineCap = 'round';
      g.shadowColor = base(0.55); g.shadowBlur = 10; g.stroke(); g.shadowBlur = 0;
      if (kind !== 'spark') {
        g.lineTo(pts[pts.length - 1][0], H - PAD); g.lineTo(pts[0][0], H - PAD); g.closePath();
        g.save(); g.clip();
        g.fillStyle = run(kind === 'area' ? 0.45 : 0.24); g.fillRect(0, 0, W, H);                     // the palette under the line ...
        const fade = g.createLinearGradient(0, 0, 0, H); fade.addColorStop(0, 'rgba(0,0,0,0)'); fade.addColorStop(1, 'rgba(0,0,0,1)');
        g.globalCompositeOperation = 'destination-out'; g.fillStyle = fade; g.fillRect(0, 0, W, H);   // ... fading out toward the bottom
        g.restore();
      }
      const [lx, ly] = pts[pts.length - 1];                      // a glowing head on the moving end
      g.beginPath(); g.arc(lx, ly, 3.2, 0, 7); g.fillStyle = ink(1); g.fill();
    }
    if (hover >= 0) { g.beginPath(); g.arc(X(hover), Y(v[hover]), 5, 0, 7); g.fillStyle = base(1); g.fill(); g.lineWidth = 2; g.strokeStyle = ink(1); g.stroke(); }
  }

  // grown on hover (#stage.grown): every bar / point shows its value, so the bigger card also tells more
  if (hover < 0 && kind !== 'spark' && n <= 14 && document.getElementById('stage').classList.contains('grown')) {
    v.forEach((val, i) => label(g, fmt(val, s.unit), X(i), Math.max(10, Y(val) - 8), { size: 9, weight: 600, color: ink(.85) }));
  }
  if (hover >= 0) {
    const txt = `${s.labels[hover] ? s.labels[hover] + ': ' : ''}${fmt(v[hover], s.unit)}`;
    label(g, txt, Math.min(W - 44, Math.max(44, X(hover))), 9, { weight: 600 });
  } else if (kind === 'bar' && s.labels.length) {
    s.labels.forEach((l, i) => label(g, l, X(i), H - 5, { size: 9.5, color: ink(.5) }));
  }
}

export function hit(s, x) {
  const n = s.values.length;
  if (!n) return -1;
  const f = s.kind === 'bar' ? (x - PAD) / (s.W - PAD * 2) * n - 0.5 : ((x - PAD) / (s.W - PAD * 2)) * (n - 1);
  return Math.max(0, Math.min(n - 1, Math.round(f)));
}

