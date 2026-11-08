// Entry point. Registers the catalog, then answers the host.

import { Renderer } from './render.js';
import { send, measureSoon, dropped, setView } from './bridge.js';
import { enter, leave, pulse, reduced, setReduced } from './motion.js';
import { pick as pickTheme, apply as applyTheme, adaptToBackdrop, setBoldness } from './theme.js';
import * as literacy from './literacy.js';
import '../components/layout.js';
import '../components/text.js';
import '../components/button.js';
import '../components/progress.js';
import '../components/stat.js';
import '../components/spinner.js';
import '../components/skeleton.js';
import '../components/input.js';
import '../components/list.js';
import '../components/chart.js';
import '../components/graph.js';

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
        r.init(m);
        if (ready) root.classList.add('on');   // refresh of an on-screen surface: stay visible
        measureSoon();
        break;
      case 'patch':
        // test hook: `morc patch <id> /__hover true|false` plays the pointer entering/leaving the card (hover-grow) without touching the user's mouse
        if (m.data && '/__hover' in m.data) { stage.dispatchEvent(new PointerEvent(m.data['/__hover'] ? 'pointerenter' : 'pointerleave')); break; }
        r.patch(m); measureSoon(); break;
      case 'backdrop': adaptToBackdrop(stage, m.rgb); break;   // sampled just before the window appears
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
