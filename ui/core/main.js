// Entry point. Registers the catalog, then answers the host.

import { Renderer } from './render.js';
import { send, measureSoon, dropped, setView, setViewSmooth, setLimits } from './bridge.js';
import { enter, leave, pulse, reduced, setReduced } from './motion.js';
import { pick as pickTheme, apply as applyTheme, adaptToBackdrop, setBoldness, setThemes } from './theme.js';

// Everything visual and every min/max is DATA (ui/tokens.json, ui/themes.json, ui/limits.json + your prefs/ui.json), sent by the host on every card and again
// whenever a file changes: CSS variables, color themes, size limits. Nothing is typed into the CSS or the code.
let lastRgb = null;
function applyUi(ui) {
  if (!ui) return;
  for (const [k, v] of Object.entries(ui.tokens || {})) document.documentElement.style.setProperty('--' + k, String(v));
  setLimits(ui.limits);
  setThemes(ui.themes);
  window.dispatchEvent(new Event('morc-zoom'));   // charts repaint with the new tokens (text floor, colors)
}
import * as literacy from './literacy.js';
// The component tree. Every component is a folder (ui/components/<name>/: code, styles, metadata.json). `morc ui build` writes ui/components.json, the list of
// those folders in dependency order; here each one's styles and code are loaded in that order, so a component always loads after what it depends on.
async function loadComponents() {
  try {
    const manifest = await (await fetch('components.json')).json();
    for (const name of manifest.order) {
      const c = manifest.components[name];
      if (c.css) {
        await new Promise((done) => {
          const l = document.createElement('link');
          l.rel = 'stylesheet';
          l.href = `${c.path}/${c.css}`;
          l.onload = l.onerror = done;
          document.head.appendChild(l);
        });
      }
      if (c.js) await import(`../${c.path}/${c.js}`).catch((e) => send({ t: 'log', msg: `component ${name}: ${e.message}` }));
    }
  } catch (e) { send({ t: 'log', msg: 'components: ' + e.message }); }
}
await loadComponents();

const stage = document.getElementById('stage');
const root = document.getElementById('surface');
const r = new Renderer(root, send);
let ready = false;

const seenErrors = new Map();   // one bug must never flood the log: the first 3 identical errors are logged, the rest counted silently
addEventListener('error', (e) => {
  if (/ResizeObserver loop/.test(e.message)) return;
  const key = `${e.message} @${e.filename}:${e.lineno}`, n = (seenErrors.get(key) || 0) + 1;
  seenErrors.set(key, n);
  if (n <= 3) send({ t: 'log', msg: key + (n === 3 ? ' (further repeats suppressed)' : '') });
});
addEventListener('unhandledrejection', (e) => send({ t: 'log', msg: 'promise: ' + e.reason }));

window.morc = {
  recv(m) {
    if (window.__debug) send({ t: 'log', msg: 'recv ' + m.t + ' ' + JSON.stringify(m.data || {}).slice(0, 120) });   // MORC_DEBUG only
    switch (m.t) {
      case 'init':
        stage.classList.toggle('urgent', !!(m.opts && m.opts.urgent));
        if (m.view) {   // the annoyance vector + importance, resolved by the host
          applyUi(m.view.ui);   // first: the limits and colors the rest of this init uses
          setReduced(m.view.motion < 0.3);
          setBoldness(m.view.boldness);
          window.__simplicity = m.view.simplicity || 0;
          window.__view = m.view;
          if (m.view.debug) send({ t: 'log', msg: 'view-debug wrap=' + m.view.wrap + ' dock_w=' + m.view.dock_w + ' morph=' + m.view.morph + ' scale=' + m.view.scale });
          window.__debug = !!m.view.debug;
          setView(m.view);
        }
        applyTheme(stage, pickTheme(m.opts && m.opts.topic, m.opts && m.opts.theme, m.view && m.view.theme_default));
        literacy.set(m.literacy);
        // card-level alignment from the card's JSON ("align": "right"): titles, rows, text, charts and Close all start from the same edge
        stage.classList.toggle('align-right', /right/.test((m.opts && m.opts.align) || ''));
        r.init(m);
        if (ready) root.classList.add('on');   // refresh of an on-screen surface: stay visible
        measureSoon();
        break;
      case 'ui':   // a token / theme / limits file changed: re-apply live, no restart
        applyUi(m.ui);
        applyTheme(stage, stage.dataset.themeName || 'coral');
        if (lastRgb) adaptToBackdrop(stage, lastRgb);
        measureSoon();
        break;
      case 'view': setViewSmooth(m.view); break;   // the host re-sizes this card (crowded screen: another card is in use)
      case 'patch':
        // test hook: `morc patch <id> /__hover true|false` plays the pointer entering/leaving the card (hover-grow) without touching the user's mouse
        // attention nudge: `morc patch <id> /__pulse true` plays the blue pulse on that card (the "look here" that costs less than opening something new)
        if (m.data && '/__pulse' in m.data) { pulse(root); break; }
        if (m.data && '/__hover' in m.data) { stage.dispatchEvent(new PointerEvent(m.data['/__hover'] ? 'pointerenter' : 'pointerleave')); break; }
        r.patch(m); measureSoon(); break;
      case 'backdrop': lastRgb = m.rgb; adaptToBackdrop(stage, m.rgb); break;   // sampled just before the window appears
      case 'enter': {
        root.classList.add('on');
        ready = true;
        stage.dataset.entered = '1';
        if (window.__debug) setTimeout(() => {   // MORC_DEBUG: what is painted near the top edge? (to log, not to screen)
          const out = [];
          for (const el of document.querySelectorAll('*')) {
            const r = el.getBoundingClientRect(), cs = getComputedStyle(el);
            const bg = cs.backgroundColor, bi = cs.backgroundImage;
            if (r.width > 0 && r.top < 60 && (bg !== 'rgba(0, 0, 0, 0)' || bi !== 'none' || cs.boxShadow !== 'none'))
              out.push(`${el.tagName}#${el.id}.${String(el.className).slice(0, 24)} [${Math.round(r.left)},${Math.round(r.top)} ${Math.round(r.width)}x${Math.round(r.height)}] bg=${bg} img=${bi.slice(0, 40)} z=${window.__zoom}`);
          }
          send({ t: 'log', msg: 'TOP-ELEMENTS ' + stage.dataset.bd + ' | ' + out.join(' || ') });
        }, 900);
        window.dispatchEvent(new Event('morc-enter'));          // charts start their intro now, not while hidden
        if (reduced) break;
        const top = root.firstElementChild;
        const kids = top ? [...top.children] : [];
        // the whole surface springs in; direct children follow in a short stagger
        enter(root);
        kids.slice(0, 8).forEach((k, i) => enter(k, 40 + i * 35));
        break;
      }
      case 'leave': leave(root).finished.then(() => send({ t: 'left' })).catch(() => send({ t: 'left' })); break;
      case 'pulse': pulse(root); break;
      case 'dropped': dropped(); break;
    }
  },
};

// Plugins: every ui/plugins/*.js is imported before the host is told we are ready, so their components exist when the first spec arrives.
async function loadPlugins() {
  try {
    const list = await (await fetch('plugins/index.json')).json();
    await Promise.all(list.map((f) => import('../plugins/' + f).catch((e) => send({ t: 'log', msg: `plugin ${f}: ${e.message}` }))));
  } catch (e) { send({ t: 'log', msg: 'plugins: ' + e.message }); }
}
loadPlugins().finally(() => send({ t: 'ready' }));
