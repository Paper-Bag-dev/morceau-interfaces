// Color themes + backdrop-aware contrast.
//   opts.theme: a name from THEMES | "random" (new each show) | omitted = "auto"
//   auto: matched to the topic (the same topic keeps its look) and shifts a little each day.
//   opts.contrast = "off" disables backdrop adaptation.
// Backdrop: the host samples the average color behind the window just before it appears. We then pick the "anti" scheme:
//   dark page (video sites, terminals)  -> light surfaces, accent = complement of the page hue, light-ish
//   light page (docs, white sites)  -> dark ink, dark surfaces, accent = complement, deeper
//   mid / colorful                  -> default dark glass, accent = complement
// An explicitly named theme keeps its accent; the light/dark scheme still adapts so text stays readable.

import { setInk } from '../charts/util.js';

// The named accents, the base color variables and the light/dark schemes are DATA in ui/themes.json (your prefs/ui.json can override them); the host sends
// them and re-sends them when the file changes (setThemes). What is below is only the fallback if that file cannot be read.
export let THEMES = {
  ocean:   '199 89% 58%',
  violet:  '262 83% 70%',
  rose:    '346 87% 66%',
  amber:   '38 92% 56%',
  emerald: '152 62% 50%',
  coral:   '14 90% 63%',
  lime:    '84 70% 52%',
  sky:     '217 91% 66%',
  fuchsia: '292 80% 68%',
  teal:    '174 72% 48%',
};

let NAMES = Object.keys(THEMES);
let CFG = { schemes: {}, 'chart-ink': { dark: '20 20 28', other: '255 255 255' } };
let schemeKeys = [];

// Which system the page runs on: `platforms.<os>` in ui/themes.json is laid over the rest there, and CSS can read html.os-<os>.
export const OS = /Mac/i.test(navigator.platform || navigator.userAgent) ? 'mac' : 'win';
document.documentElement.classList.add('os-' + OS);

export function setThemes(t) {
  if (!t || typeof t !== 'object') return;
  // layers, later wins: the defaults, then the skin (a named material set), then this platform's own values
  const p = (t.platforms || {})[OS] || {};
  const skin = p.skin ?? t.skin;
  // a skin can be several words ("glass flat"): each named skin's data is laid on in that order
  const sks = String(skin || '').split(/\s+/).filter(Boolean).map((w) => (t.skins || {})[w] || {});
  const sk = { base: Object.assign({}, ...sks.map((x) => x.base || {})), 'chart-ink': Object.assign({}, ...sks.map((x) => x['chart-ink'] || {})) };
  const schemes = { ...(t.schemes || {}) };
  for (const src of [...sks.map((x) => x.schemes), p.schemes]) for (const [k, v] of Object.entries(src || {})) schemes[k] = { ...(schemes[k] || {}), ...v };
  t = { ...t, skin, accents: { ...(t.accents || {}), ...(p.accents || {}) }, default: p.default || t.default,
        base: { ...(t.base || {}), ...(sk.base || {}), ...(p.base || {}) }, schemes,
        'chart-ink': { ...(t['chart-ink'] || {}), ...sk['chart-ink'], ...(p['chart-ink'] || {}) } };
  // CSS keys off html[data-skin] (none = the default plates)
  if (skin) document.documentElement.dataset.skin = skin; else delete document.documentElement.dataset.skin;
  if (t.accents && Object.keys(t.accents).length) { THEMES = t.accents; NAMES = Object.keys(THEMES); }
  if (t.default) DEFAULT_THEME = t.default;
  CFG = { ...CFG, ...t };
  for (const [k, v] of Object.entries(t.base || {})) document.documentElement.style.setProperty('--' + k, String(v));
}

function hash(s) {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 16777619); }
  return h >>> 0;
}

// Default is ONE consistent accent (coral, the user's pick 11-08) so pin, dismiss, buttons and charts always match.
// Variety only when asked: opts.theme a name | 'random' | 'anti' (complement of the page), or the vector's theme = 'auto' (topic + day).
export let DEFAULT_THEME = 'coral';

export function pick(topic, theme, dflt) {
  if (theme === 'anti') return 'anti';
  if (theme && THEMES[theme]) return theme;
  if (theme === 'random') return NAMES[Math.floor(Math.random() * NAMES.length)];
  if (dflt === 'auto') {
    const day = new Date().toISOString().slice(0, 10);
    return NAMES[hash(`${topic || 'adhoc'}|${day}`) % NAMES.length];
  }
  return THEMES[dflt] ? dflt : DEFAULT_THEME;
}

/** Apply a theme name's accent (the backdrop step may refine it). */
export function apply(el, name) {
  el.dataset.themeName = name;
  const hsl = THEMES[name];
  if (hsl) setAccent(el, hsl);
  else setAccent(el, THEMES.sky);
  el.dataset.explicit = name === 'anti' ? '' : '1';   // only 'anti' lets the backdrop recolor the accent
  setScheme(el, 'mid');
}

let boldness = 1;
/** Annoyance vector `boldness`: scales accent saturation (0.3 = nearly grey, 1.2 = punchy). */
export function setBoldness(b) { boldness = Number.isFinite(b) ? b : 1; }

function setAccent(el, hsl) {
  const [hh, ss, ll] = hsl.split(/\s+/);
  hsl = `${hh} ${Math.min(100, Math.round(parseFloat(ss) * boldness))}% ${ll}`;
  el.style.setProperty('--accent', hsl);
  el.style.setProperty('--ring', hsl);
  const l = parseFloat(hsl.split(/\s+/)[2]);
  el.style.setProperty('--on-accent', l > 52 ? '240 10% 8%' : '0 0% 100%');   // readable text on the accent
  // the palette as CSS variables (--pal-1 .. --pal-6), the same walk the charts use, so any component can give each of its items a color
  const [h0, s0, l0] = hsl.split(/\s+/).map(parseFloat), b = CFG.base || {};
  const hs = parseFloat(b['chart-hue-spread']), lsp = parseFloat(b['chart-light-spread']);
  for (let i = 0; i < 6; i++) {
    const t = i / 5, hue = (h0 + t * (Number.isFinite(hs) ? hs : 70) + 360) % 360;
    const light = Math.max(18, Math.min(86, l0 + (t - 0.5) * (Number.isFinite(lsp) ? lsp : 0)));
    el.style.setProperty(`--pal-${i + 1}`, `${Math.round(hue)} ${s0}% ${Math.round(light)}%`);
    // the deep twin for filled controls (Yes / No): calmer saturation and darker, so a big filled area sits with the dark glass (themes: pal-deep-sat, pal-deep-light)
    const ds = parseFloat(b['pal-deep-sat']), dl = parseFloat(b['pal-deep-light']);
    el.style.setProperty(`--pal-${i + 1}-deep`, `${Math.round(hue)} ${Math.round(s0 * (Number.isFinite(ds) ? ds : 0.62))}% ${Math.round(Math.max(18, light - (Number.isFinite(dl) ? dl : 12)))}%`);
  }
}

function setScheme(el, bd) {
  el.dataset.bd = bd;
  // ink for chart labels/grids drawn on canvas sits on the card surface: light cards need dark ink and vice versa
  const ink = CFG['chart-ink'] || {};
  setInk(bd === 'dark' ? (ink.dark || '20 20 28') : (ink.other || '255 255 255'));
  // the scheme's color variables (from ui/themes.json) go on the card; the previous scheme's are cleared first
  for (const k of schemeKeys) el.style.removeProperty('--' + k);
  schemeKeys = Object.keys((CFG.schemes && CFG.schemes[bd]) || {});
  for (const k of schemeKeys) el.style.setProperty('--' + k, String(CFG.schemes[bd][k]));
}

function rgbToHsl([r, g, b]) {
  r /= 255; g /= 255; b /= 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
  let h = 0;
  if (d) {
    if (mx === r) h = ((g - b) / d) % 6; else if (mx === g) h = (b - r) / d + 2; else h = (r - g) / d + 4;
    h = (h * 60 + 360) % 360;
  }
  const l = (mx + mn) / 2, s = d ? d / (1 - Math.abs(2 * l - 1)) : 0;
  return { h, s, l };
}

/** rgb = average color behind the window. */
export function adaptToBackdrop(el, rgb) {
  const { h, s, l } = rgbToHsl(rgb);
  const lum = (0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]) / 255;
  const bd = lum < 0.34 ? 'dark' : lum > 0.62 ? 'light' : 'mid';
  setScheme(el, bd);
  // anti-color accent: opposite hue; if the page is nearly grey, keep a lively default hue instead of an arbitrary one
  if (el.dataset.explicit !== '1') {
    const hue = s < 0.12 ? (hashHue(el.dataset.themeName) ) : (h + 180) % 360;
    const light = bd === 'dark' ? 64 : bd === 'light' ? 42 : 58;
    setAccent(el, `${Math.round(hue)} ${bd === 'mid' ? 78 : 84}% ${light}%`);
  }
  el.dataset.lum = lum.toFixed(2);
}

function hashHue(name) {
  return hash(name || 'x') % 360;
}
