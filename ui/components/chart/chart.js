import { registerView, useLayoutEffect } from '../../core/view.js';
import { reduced } from '../../core/motion.js';
import * as literacy from '../../core/literacy.js';
import { working } from '../../core/working.js';
import { EXPLAIN, FANCY } from '../../charts/explain.js';
import * as cartesian from '../../charts/cartesian.js';
import * as radar from '../../charts/radar.js';
import * as donut from '../../charts/donut.js';
import * as gauge from '../../charts/gauge.js';
import * as heatmap from '../../charts/heatmap.js';
import * as clock from '../../charts/clock.js';
import * as rings from '../../charts/rings.js';
import * as treemap from '../../charts/treemap.js';

// {type:"chart", kind:"line|area|spark|bar|radar|donut|gauge|heatmap|clock|rings|treemap",
//  title, data:{bind}, labels:{bind}, unit (string or {bind}), max, targets:[...], cols (heatmap), height}
// Pick the kind by the question (docs/charts.md). Unfamiliar kinds (see charts/explain.js) explain themselves the first
// couple of times; "Still confusing" swaps in a plain chart and tells the agent, so the approach adapts.

const MODS = { line: cartesian, area: cartesian, spark: cartesian, bar: cartesian, radar, donut, gauge, heatmap, clock, rings, treemap };
const HEIGHT = { gauge: 110, radar: 160, donut: 130, clock: 170, rings: 150, treemap: 150, heatmap: 110 };
const INTRO_MS = 950;
const SIMPLER = 'Same numbers, simpler chart: higher means more. Tell me what was unclear and I will explain it another way.';

// Preact view (docs/views.md). Drawing is imperative by nature (a canvas), so the view mounts it once on its own host,
// synchronously at the first commit, so the card measures it like before.
function mount(c, ctx, el) {
  const ttl = document.createElement('div');
  ttl.className = 'ttl';
  ctx.bind(c.title, (v) => { ttl.textContent = v ?? ''; });
  const cv = document.createElement('canvas');
  // key + detail: the canvas sits in a sideways fold (.fold-x). A chart module that knows its key part (keyWidth(s): the donut's ring)
  // lets a compact card show just that; hovering slides the rest (the legend) out to the right (core/compact.js, plate.css)
  const fold = document.createElement('div');
  fold.className = 'fold-x';
  fold.append(cv);
  el.append(ttl, fold);

  let kind = MODS[c.kind] ? c.kind : 'line';
  const s = { c, kind, W: 260, H: 90, values: [], target: [], labels: [], hover: -1, unit: typeof c.unit === 'string' && !c.unit.includes('{') ? c.unit : '', t: reduced ? 1 : 0 };
  // Fixed pixel width: a %-width canvas inside a shrink-wrapped surface feeds back on itself and grows every repaint.
  s.W = c.width || 280;
  cv.style.width = s.W + 'px';
  const sizeH = () => { s.H = c.height || HEIGHT[kind] || 90; cv.style.height = s.H + 'px'; fitW(); };
  // the chart sizes itself from its labels (a module may export width(s)): the canvas never clips its own legend, and the window follows
  function fitW() {
    const need = MODS[kind] && MODS[kind].width ? MODS[kind].width(s) : 0;
    // a chart that knows its own width (donut: ring + legend) is exactly as wide as that, never padded out to a default;
    // the others keep the default width. The card hugs its content instead of leaving an empty white strip that the window can clip.
    const full = need ? Math.max(c.width || 0, need) : (c.width || 280);
    const key = MODS[kind] && MODS[kind].keyWidth ? MODS[kind].keyWidth(s) : 0;
    if (key && key < full) { fold.style.setProperty('--key-w', key + 'px'); fold.dataset.key = ''; } else { delete fold.dataset.key; }
    // folded (compact, not opening): the canvas really is only the key part, so the card hugs it and the detail is not drawn at all
    const st = document.getElementById('stage');
    const w = key && key < full && st.classList.contains('compact') && !st.dataset.opening ? key : full;
    if (w !== s.W) { s.W = w; cv.style.width = w + 'px'; }
  }
  sizeH();

  // ---- teaching caption for unfamiliar kinds ----
  let cap = null;
  function showCaption() {
    const lit = literacy.get(c.kind);
    const confused = lit.confused > 0 && lit.got === 0;
    if (!FANCY.has(c.kind) || lit.got > 0 || (!confused && lit.seen >= 2)) return;
    cap = document.createElement('div');
    cap.className = 'explain';
    const txt = document.createElement('div');
    txt.className = 'txt';
    txt.textContent = confused ? SIMPLER : EXPLAIN[c.kind].short;
    const chips = document.createElement('div');
    chips.className = 'chips';
    const mk = (label, fn) => { const b = document.createElement('button'); b.className = 'mini'; b.textContent = label; b.addEventListener('click', fn); return b; };
    const done = () => { cap.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 160 }).finished.then(() => cap.remove(), () => cap.remove()); };
    if (confused) chips.append(mk('OK', done));
    else {
      chips.append(
        mk('Got it', () => { literacy.mark(c.kind, 'got'); done(); }),
        mk('Still confusing', () => {
          literacy.mark(c.kind, 'confused');
          txt.textContent = SIMPLER;
          kind = s.values.length <= 12 ? 'bar' : 'line'; s.kind = kind; sizeH(); s.t = reduced ? 1 : 0; start();
          chips.replaceChildren(mk('OK', done));
        }),
      );
    }
    cap.append(txt, chips);
    el.append(cap);
    if (!confused) literacy.mark(c.kind, 'seen');
    if (confused) { kind = s.values.length <= 12 ? 'bar' : 'line'; s.kind = kind; sizeH(); }
  }

  // ---- drawing + animation ----
  let raf = 0, t0 = 0, moving = false;
  function paint() {
    const dpr = (devicePixelRatio || 1) * (window.__zoom || 1);   // stay crisp when the surface is scaled up
    cv.width = s.W * dpr; cv.height = s.H * dpr;
    const g = cv.getContext('2d');
    g.scale(dpr, dpr);
    g.clearRect(0, 0, s.W, s.H);
    s.kind = kind;
    MODS[kind].draw(g, s);
  }
  function frame(now) {
    s.t = Math.min(1, (now - t0) / INTRO_MS);
    moving = false;
    s.values = s.values.map((v, i) => { const d = (s.target[i] ?? 0) - v; if (Math.abs(d) < 0.01) return s.target[i] ?? 0; moving = true; return v + d * 0.16; });
    paint();
    raf = s.t < 1 || moving ? requestAnimationFrame(frame) : 0;
  }
  const busy = (c.height || 0) >= 110 || ['clock', 'rings', 'treemap', 'radar', 'donut'].includes(c.kind) ? working(el) : { done() {} };   // large charts: blue pulse until they start drawing
  function start() { busy.done(); cancelAnimationFrame(raf); t0 = performance.now(); raf = requestAnimationFrame(frame); }
  const kick = () => { if (!raf) raf = requestAnimationFrame(frame); };

  function setData(v) {
    const flat = Array.isArray(v) ? v.flat() : v == null ? [] : [v];
    s.target = flat.map(Number).map((n) => (Number.isFinite(n) ? n : 0));
    const fresh = !s.values.length;
    while (s.values.length < s.target.length) s.values.push(fresh ? s.target[s.values.length] : (s.values[s.values.length - 1] ?? 0));
    s.values.length = s.target.length;
    if (fresh) {
      // annoyance vector: when `simplicity` is high, unfamiliar kinds quietly become plain ones (no caption needed)
      if (FANCY.has(c.kind) && (window.__simplicity || 0) > 0.7) { kind = s.values.length <= 12 ? 'bar' : 'line'; s.kind = kind; sizeH(); } else showCaption();
      // start the intro when the surface actually appears, not while it is still hidden
      const go = () => start();
      document.getElementById('stage').dataset.entered ? go() : addEventListener('morc-enter', go, { once: true });
    } else kick();
  }

  ctx.bind(c.data, setData);
  // a {bind} or a "{/unit}" template resolves like every other prop (a template used to be drawn literally: "198{/unit}")
  if (c.unit && (typeof c.unit === 'object' || c.unit.includes('{'))) ctx.bind(c.unit, (v) => { s.unit = v ?? ''; kick(); });
  if (c.labels) ctx.bind(c.labels, (v) => { s.labels = Array.isArray(v) ? v : []; fitW(); kick(); });
  cv.addEventListener('pointermove', (e) => {
    const z = window.__zoom || 1, r = cv.getBoundingClientRect(), h = MODS[kind].hit(s, (e.clientX - r.left) / z, (e.clientY - r.top) / z);
    if (h !== s.hover) { s.hover = h; if (!raf) paint(); }
  });
  cv.addEventListener('pointerleave', () => { s.hover = -1; if (!raf) paint(); });
  const onZoom = () => { if (!el.isConnected) return removeEventListener('morc-zoom', onZoom); fitW(); if (!raf && s.values.length) paint(); };   // fitW: a token (text floor) may have changed the width the labels need
  addEventListener('morc-zoom', onZoom);
}

registerView('chart', ({ c, ctx, host }) => {
  useLayoutEffect(() => { mount(c, ctx, host); }, []);
  return null;
}, { cls: 'c-chart' });

