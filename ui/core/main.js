// Entry point. Registers the catalog, then answers the host.

import { Renderer } from './render.js';
import { send, measureSoon, dropped, setLayer, setView, setViewSmooth, setLimits, limit, followGlass } from './bridge.js';
import { working } from './working.js';
import { enter, arrive, leave, pulse, reduced, setReduced } from './motion.js';
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
import { cardTree } from './tree.js';
import { applyRemote, seedShared } from './sharedbind.js';
import { twReady } from './view.js';   // Preact views, the "el" component and Tailwind classes (docs/views.md)

// The card owns its width (docs/views.md, layout rule 5): once laid out, the root's width is fixed, so a data patch (a long value) wraps
// inside the card instead of widening it. A structural change (new components, a refresh, a new size from the host) unlocks it, the
// card is measured once more and locked again. A spec that sets its own width ("class": "w-[...]", or width) keeps it.
let lockTimer = 0;
function unlockWidth() {
  const el = root.firstElementChild;
  if (el && el.dataset.wlock) { el.style.width = ''; el.style.minWidth = ''; delete el.dataset.wlock; }
}
function lockWidthSoon() {
  clearTimeout(lockTimer);
  lockTimer = setTimeout(async () => {
    await document.fonts.ready;
    await twReady();
    requestAnimationFrame(() => requestAnimationFrame(() => {
      const el = root.firstElementChild;
      if (!el || el.dataset.wlock) return;
      const spec = r.comps.get(el.dataset.cid) || {};
      if (/(^|\s)w-/.test(spec.class || '') || spec.width) return;
      if (!document.querySelector('#surface [data-hit]')) return lockWidthSoon();   // still empty (a streamed build): try again
      el.style.width = getComputedStyle(el).width;
      el.style.minWidth = 'min-content';   // min-width beats width: text wraps inside the lock, a rigid child (a chart that grows its legend) still widens the card
      el.dataset.wlock = '1';
    }));
  }, 400);
}

// opts.persist: send the card's data to the host with a falloff: 1 s after the last change, and at most every 5 s while changes keep
// coming (a value streaming in, a slider held down), so a busy card is one write per 5 s, never one per change (host/state.rs writes it).
let snapTimer = 0, snapFirst = 0;
function persist(on) {
  const flush = () => { clearTimeout(snapTimer); snapFirst = 0; send({ t: 'snap', data: r.store.data }); };
  r.store.onChange = on ? () => {
    const now = Date.now();
    if (!snapFirst) snapFirst = now;
    clearTimeout(snapTimer);
    if (now - snapFirst >= 5000) flush(); else snapTimer = setTimeout(flush, 1000);
  } : null;
}
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
        stage.classList.toggle('pulse', !!(m.opts && m.opts.pulse));
        stage.dataset.size = (m.opts && m.opts.size) || '';   // compact | medium | large (core/bridge.js COMPACT)
        stage.dataset.idle = (m.opts && m.opts.idle) || '';   // 0..1: rest at this fraction when not in use (core/bridge.js IDLE)
        delete stage.dataset.tall;   // how much this card has to show is measured again for the new content   // motion only for importance high/critical; urgent alone is a still ring
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
        window.__cardId = m.id;
        unlockWidth();
        if (m.shared) seedShared(m.shared);   // unlinked cards: the host's copy of the shared store
        r.init(m);
        lockWidthSoon();
        persist(!!(m.opts && m.opts.persist));
        if (ready) root.classList.add('on');   // refresh of an on-screen surface: stay visible
        measureSoon();
        break;
      case 'ui':   // a token / theme / limits file changed: re-apply live, no restart
        applyUi(m.ui);
        applyTheme(stage, stage.dataset.themeName || 'coral');
        if (lastRgb) adaptToBackdrop(stage, lastRgb);
        measureSoon();
        break;
      case 'align':   // the host placed this card: contents start at the edge nearest the active window (left of it: right-aligned, otherwise left-aligned)
        stage.classList.toggle('align-right', !!m.right);
        measureSoon();
        break;
      case 'view': unlockWidth(); setViewSmooth(m.view); lockWidthSoon(); break;   // the host re-sizes this card (crowded screen: another card is in use)
      case 'patch':
        // test hook: `morc patch <id> /__hover true|false` plays the pointer entering/leaving the card (hover-grow) without touching the user's mouse
        // attention nudge: `morc patch <id> /__pulse true` plays the blue pulse on that card (the "look here" that costs less than opening something new)
        if (m.data && '/__pulse' in m.data) { const w = working(root); pulse(root); setTimeout(() => w.done(), limit('pulse-ms', 1600)); break; }   // the blue system pulse, then one ring
        // test hook: `morc patch <id> /__controls true|false` plays the pointer reaching the corner (the card's buttons come out)
        if (m.data && '/__controls' in m.data) { document.getElementById('controls').classList.toggle('hov', !!m.data['/__controls']); break; }
        if (m.data && '/__hover' in m.data) { stage.dispatchEvent(new PointerEvent(m.data['/__hover'] ? 'pointerenter' : 'pointerleave')); break; }
        { const structural = m.components && m.components.length; if (structural) unlockWidth(); r.patch(m); if (structural) lockWidthSoon(); }
        measureSoon(); break;
      // the host's tree query (morc tree): answer with this card's tree (core/tree.js)
      case 'tree': {
        let body;
        try { body = { ok: true, tree: cardTree(r, m) }; } catch (e) { body = { ok: false, err: 'tree: ' + e.message }; }
        send({ t: 'reply', rid: m.rid, body });
        break;
      }
      // morc do <card> <action> [args]: a named action (opts.actions); the answer carries the card's data after it ran
      case 'do': {
        let body;
        try { r.call(m.name, m.args); body = { ok: true, data: r.store.data }; } catch (e) { body = { ok: false, err: e.message }; }
        send({ t: 'reply', rid: m.rid, body });
        measureSoon();
        break;
      }
      case 'refresh': refresh(m.names); break;
      case 'layer': setLayer(m.below); break;   // the host moved this card behind / in front of the active window
      case 'shared': applyRemote(m.path, m.value); break;   // unlinked cards: another card (or morc patch @shared) changed the shared store
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
        if (reduced) { followGlass(null); break; }
        const top = root.firstElementChild;
        const kids = top ? [...top.children] : [];
        // the card appears from nothing and springs up to size (the glass grows with it); its children follow in a short stagger
        followGlass(arrive(root, limit));
        kids.slice(0, 8).forEach((k, i) => enter(k, 40 + i * 35));
        // a new card glows softly for a moment (limits arrive-pulse-ms; 0 = off), then stays still
        if (limit('arrive-pulse-ms', 0) > 0) { stage.classList.add('arrive'); setTimeout(() => stage.classList.remove('arrive'), limit('arrive-pulse-ms', 0)); }
        break;
      }
      case 'leave': { send({ t: 'final', data: r.store.data }); const a = leave(root, limit); followGlass(a); a.finished.then(() => send({ t: 'left' })).catch(() => send({ t: 'left' })); break; }   // the exit mirrors the arrival; the glass follows it
      case 'pulse': pulse(root); break;
      // macOS: the host's hit test saw the pointer go onto / off the card (the web view cannot while our app is inactive)
      case 'hover': stage.dispatchEvent(new PointerEvent(m.on ? 'pointerenter' : 'pointerleave')); if (!m.on) markHover(null); break;
      // macOS: where the pointer is on the card; every hover style keys off :hover OR .hov, so rows, buttons and pills react without a click
      case 'pointer': { const el = document.elementFromPoint(m.x, m.y); markHover(el); (el || document).dispatchEvent(new PointerEvent('pointermove', { clientX: m.x, clientY: m.y, bubbles: true })); break; }
      case 'dropped': dropped(); break;
    }
  },
};

// "user/x.js" = one of your plugins (<home>/plugins, served at user-plugins/); anything else is in ui/plugins.
const pluginUrl = (f, v) => (f.startsWith('user/') ? '../user-plugins/' + f.slice(5) : '../plugins/' + f) + (v ? '?v=' + v : '');

// morc refresh [names]: import the named components / plugins again under a new URL (each re-registers its type), swap their stylesheets,
// then rebuild the card from its store. The data stays; only what the DOM held (scroll, focus) is rebuilt.
async function refresh(names) {
  const v = Date.now().toString(36), want = (n) => !names || !names.length || names.includes(n);
  try {
    const manifest = await (await fetch('components.json')).json();
    for (const name of manifest.order.filter(want)) {
      const c = manifest.components[name];
      if (c.css) {
        const old = [...document.querySelectorAll('link[rel=stylesheet]')].find((l) => l.href.includes(`${c.path}/${c.css}`));
        if (old) old.href = `${c.path}/${c.css}?v=${v}`;
      }
      if (c.js) await import(`../${c.path}/${c.js}?v=${v}`).catch((e) => send({ t: 'log', msg: `refresh ${name}: ${e.message}` }));
    }
    const list = await (await fetch('plugins/index.json')).json();
    for (const f of list.filter((f) => want(f) || want(f.replace(/^user\//, '')) || want(f.replace(/^user\//, '').replace(/\.js$/, ''))))
      await import(pluginUrl(f, v)).catch((e) => send({ t: 'log', msg: `refresh ${f}: ${e.message}` }));
  } catch (e) { send({ t: 'log', msg: 'refresh: ' + e.message }); }
  r.build();
  lockWidthSoon();
  measureSoon();
}

// Plugins: every ui/plugins/*.js is imported before the host is told we are ready, so their components exist when the first spec arrives.
async function loadPlugins() {
  try {
    const list = await (await fetch('plugins/index.json')).json();
    await Promise.all(list.map((f) => import(pluginUrl(f)).catch((e) => send({ t: 'log', msg: `plugin ${f}: ${e.message}` }))));
  } catch (e) { send({ t: 'log', msg: 'plugins: ' + e.message }); }
}
loadPlugins().finally(() => send({ t: 'ready' }));

// .hov = "the pointer is here" when the page cannot see the pointer itself (macOS, our app inactive). The element under it and all its
// ancestors get it, exactly like :hover; the previous chain loses it.
let hovChain = [];
function markHover(el) {
  const next = [];
  for (let e = el; e && e !== document.documentElement; e = e.parentElement) next.push(e);
  for (const e of hovChain) if (!next.includes(e)) { e.classList.remove('hov'); e.dispatchEvent(new PointerEvent('pointerleave')); }   // e.g. a chart clears its hover
  for (const e of next) e.classList.add('hov');
  hovChain = next;
}
