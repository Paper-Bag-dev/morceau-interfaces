// Shared chart helpers: accent-derived palette and easing.

export function accent() {
  const v = getComputedStyle(document.getElementById('stage')).getPropertyValue('--accent').trim() || '217 91% 66%';
  const [h, s, l] = v.replace(/%/g, '').split(/\s+/).map(Number);
  return { h, s, l };
}

/** i-th of n colors, walking around the hue wheel from the accent (so one theme gives a family, not a rainbow). */
export function palette(i, n, alpha = 1, spread = 70) {
  const { h, s, l } = accent();
  const hue = (h + (n > 1 ? (i / (n - 1)) * spread : 0) + 360) % 360;
  return `hsl(${hue} ${s}% ${l}% / ${alpha})`;
}

export const base = (alpha = 1) => { const { h, s, l } = accent(); return `hsl(${h} ${s}% ${l}% / ${alpha})`; };

// Ink for text/grids drawn on the chart canvas (which sits on the card surface). core/theme.js sets it per scheme.
let INK = '255 255 255';
export const setInk = (rgb) => { INK = rgb; };
export const ink = (a = 1) => `rgb(${INK} / ${a})`;

export const easeOutCubic = (t) => 1 - Math.pow(1 - t, 3);
/** Overshoots slightly then settles: the "popping" feel. */
export const easeOutBack = (t) => { const c1 = 1.5, c3 = c1 + 1; return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2); };
export const clamp01 = (t) => Math.max(0, Math.min(1, t));
/** Staggered sub-progress: item i of n starts a little later. */
export const stagger = (t, i, n, spread = 0.5) => clamp01((t - (i / Math.max(1, n)) * spread) / (1 - spread));

export const fmt = (v, unit = '') => `${Math.round(v * 10) / 10}${unit}`;

const BODY = '"Segoe UI Variable Text", "Segoe UI", sans-serif';
const DISPLAY = '"Bahnschrift", "Segoe UI Variable Display", "Segoe UI", sans-serif';   // numerals: DIN-like, matches the stat component

export const FLOOR = 13;   // smallest chart text in css px; bigger text is passed explicitly

export function label(g, text, x, y, { align = 'center', size = 11, color, weight = 500, display = false } = {}) {
  g.font = `${weight} ${Math.max(FLOOR, size)}px ${display ? DISPLAY : BODY}`;   // type floor: chart text is never smaller than FLOOR
  g.fontVariantNumeric = 'tabular-nums';
  g.fillStyle = color || ink(0.85);
  g.textAlign = align;
  g.textBaseline = 'middle';
  g.fillText(text, x, y);
}

// Width in css px of a text at a size and weight, in the chart body font: charts size themselves from their labels instead of clipping them.
let mctx = null;
export function measure(text, size = 11, weight = 500) {
  if (!mctx) mctx = document.createElement('canvas').getContext('2d');
  mctx.font = `${weight} ${Math.max(FLOOR, size)}px ${BODY}`;
  return mctx.measureText(String(text)).width;
}
