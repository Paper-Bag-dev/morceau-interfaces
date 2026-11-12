// This card's tree (docs/trees.md), built on request from the component list and the DOM: what the host's scene tree mounts under the card.
//   {card, size, zoom, data, shared, actions, root: {id, type, rect [x,y,w,h] css px, flags?, issues?, text?, children? | more}}
// Flags say what a screenshot would have shown: cut (text cut off sideways), clipped (content hidden with no scroll), scrolls (reachable
// by scrolling), folded (a compact card resting small; opens on hover), offcard (sticks out of the window), hidden (not shown now,
// e.g. a `when`), empty (a leaf that shows nothing: no text, picture or painted background),
// not-rendered (in the spec, not in the DOM).

import { shared } from './sharedbind.js';

const HIDE = new Set(['hidden', 'clip']);
const SCROLL = new Set(['auto', 'scroll']);

function snippet(e) {
  const t = (e.innerText || e.textContent || '').replace(/\s+/g, ' ').trim();
  return t.length > 60 ? t.slice(0, 57) + '...' : t;
}

// Checks this component's own elements (not those of nested components, which get their own node).
function check(el, b, vw, vh) {
  const flags = new Set(), issues = [];
  const cs0 = getComputedStyle(el);
  if (cs0.display === 'none' || cs0.visibility === 'hidden' || (b.width === 0 && b.height === 0)) return { flags: ['hidden'], issues };
  if (b.right > vw + 1 || b.bottom > vh + 1 || b.left < -1 || b.top < -1) flags.add('offcard');
  for (const e of [el, ...el.querySelectorAll('*')]) {
    if (e !== el && e.closest('[data-cid]') !== el) continue;
    if (!e.clientWidth && !e.clientHeight) continue;
    const cs = getComputedStyle(e);
    const text = () => (e.textContent || '').trim();
    if (e.scrollWidth > e.clientWidth + 1 && (HIDE.has(cs.overflowX) || cs.textOverflow === 'ellipsis') && text()) {
      flags.add('cut'); if (issues.length < 3) issues.push({ flag: 'cut', text: snippet(e) });
    }
    if (e.scrollHeight > e.clientHeight + 1) {
      if (SCROLL.has(cs.overflowY)) flags.add('scrolls');
      // a card resting folded (compact: it springs open on hover) hides rows on purpose: that is `folded`, reachable, not `clipped`
      else if (HIDE.has(cs.overflowY) && text()) {
        const f = document.getElementById('stage').classList.contains('compact') ? 'folded' : 'clipped';
        flags.add(f); if (issues.length < 3) issues.push({ flag: f, text: snippet(e) });
      }
    }
  }
  return { flags: [...flags], issues };
}

// shows something without text or a picture: a painted background (a colour swatch, a skeleton bar) on it or inside it
function painted(el) {
  for (const e of [el, ...el.querySelectorAll('*')]) {
    if (!e.clientWidth || !e.clientHeight) continue;
    const cs = getComputedStyle(e);
    if (cs.backgroundImage !== 'none') return true;
    const m = cs.backgroundColor.match(/[\d.]+/g);
    if (m && (m.length < 4 || +m[3] > 0.05)) return true;
  }
  return false;
}

function summary(data) {
  const out = {};
  for (const [k, v] of Object.entries(data || {})) {
    out[k] = Array.isArray(v) ? `array(${v.length})` : v && typeof v === 'object' ? `object(${Object.keys(v).length})`
      : typeof v === 'string' && v.length > 40 ? v.slice(0, 37) + '...' : v;
  }
  return out;
}

export function cardTree(r, { depth, node } = {}) {
  const maxD = depth == null ? 64 : depth;
  const vw = innerWidth, vh = innerHeight;
  const rootId = node || (r.comps.has('root') ? 'root' : [...r.comps.keys()][0]);
  const walk = (id, d) => {
    const c = r.comps.get(id);
    if (!c) return { id, flags: ['missing'] };
    const n = { id, type: c.type };
    const el = r.root.querySelector(`[data-cid="${CSS.escape(id)}"]`);
    if (!el) n.flags = ['not-rendered'];
    else {
      // which pipe built it: "preact" (registerView, docs/views.md) or "dom" (a hand-built component still to move onto it)
      n.view = el.dataset.view || 'dom';
      const b = el.getBoundingClientRect();
      n.rect = [b.x, b.y, b.width, b.height].map(Math.round);
      const { flags, issues } = check(el, b, vw, vh);
      if (flags.length) n.flags = flags;
      if (issues.length) n.issues = issues;
      if (!(c.children || []).length) {
        const t = snippet(el);
        if (t) n.text = t;
        // a leaf showing nothing at all: no text and no picture (canvas, svg, img, video)
        else if (!flags.includes('hidden') && !el.querySelector('canvas,svg,img,video,input,textarea') && !painted(el)) n.flags = [...(n.flags || []), 'empty'];
      }
    }
    const kids = c.children || [];
    if (kids.length) { if (d < maxD) n.children = kids.map((k) => walk(k, d + 1)); else n.more = kids.length; }
    return n;
  };
  const sh = shared();
  return { card: window.__cardId, size: [vw, vh], zoom: window.__zoom || 1, data: summary(r.store.data),
    shared: sh.linked ? sh.keys() : null, actions: Object.keys(r.actions || {}),
    // who owns the card's width: "locked" (the engine fixed it after layout), "spec" (a w-[...] class / width), or "content" (not yet)
    width: (() => { const el = r.root.firstElementChild; const c = el && r.comps.get(el.dataset.cid); return el && el.dataset.wlock ? 'locked' : c && (/(^|\s)w-/.test(c.class || '') || c.width) ? 'spec' : 'content'; })(),
    // what the card itself is doing: hovered (the pointer, or the host's .hov mark on macOS), pinned, lowered in the window stack
    state: (() => { const st = document.getElementById('stage'); return st && { hovered: st.matches(':hover'), hov: st.classList.contains('hov'), pinned: st.classList.contains('pinned'), below: st.classList.contains('below') }; })(),
    // the card's own buttons: where they are and whether they show (css px, unzoomed page coordinates)
    controls: [...document.querySelectorAll('#controls .ctl')].map((b) => { const q = b.getBoundingClientRect(); return { id: b.id, rect: [q.x, q.y, q.width, q.height].map(Math.round), opacity: +getComputedStyle(b).opacity }; }),
    root: rootId ? walk(rootId, 0) : null };
}
