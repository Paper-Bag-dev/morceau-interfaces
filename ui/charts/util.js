// Shared chart helpers: accent-derived palette and easing.

export function accent() {
  const v = getComputedStyle(document.getElementById('stage')).getPropertyValue('--accent').trim() || '217 91% 66%';
  const [h, s, l] = v.replace(/%/g, '').split(/\s+/).map(Number);
  return { h, s, l };
}

/** A number from the theme (a CSS variable on the card), or the fallback. */
export function themeNum(name, fallback) {
  const v = parseFloat(getComputedStyle(document.getElementById('stage')).getPropertyValue(name));
  return Number.isFinite(v) ? v : fallback;
}

/** i-th of n colors: a family made from the accent. How far it walks round the hue wheel and how much it steps in lightness are theme data
 *  (themes.json: chart-hue-spread in degrees, chart-light-spread in % points; the glass skin keeps one hue and steps lightness), so every
 *  chart, graph and future component follows the theme and a new accent recolors them all. `spread` is only the fallback. */
export function palette(i, n, alpha = 1, spread = 70) {
  const { h, s, l } = accent();
  const t = n > 1 ? i / (n - 1) : 0;
  const hue = (h + t * themeNum('--chart-hue-spread', spread) + 360) % 360;
  const ls = themeNum('--chart-light-spread', 0);
  const light = Math.max(18, Math.min(86, l + (t - 0.5) * ls));
  return `hsl(${hue} ${s}% ${light}% / ${alpha})`;
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

// smallest chart text in css px: the token `chart-text-floor` (ui/tokens.json, overridable in prefs/ui.json); bigger text is passed explicitly
export const floor = () => parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--chart-text-floor')) || 13;

export function label(g, text, x, y, { align = 'center', size = 11, color, weight = 500, display = false } = {}) {
  g.font = `${weight} ${Math.max(floor(), size)}px ${display ? DISPLAY : BODY}`;   // type floor: chart text is never smaller than FLOOR
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
  mctx.font = `${weight} ${Math.max(floor(), size)}px ${BODY}`;
  return mctx.measureText(String(text)).width;
}
