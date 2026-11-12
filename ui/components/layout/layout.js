import { register } from '../../core/registry.js';
import { registerView, viewFactory, html, useLayoutEffect } from '../../core/view.js';

// Layouts are Preact views whose host takes the renderer's child components (slot); the view sets the host's gap, align, panel and scroll.
const box = (cls) => ({ c, host }) => {
  if (c.gap != null) host.style.setProperty('--gap', c.gap + 'px');
  if (c.align) host.style.setProperty('--align', c.align);
  if (c.scroll) host.style.maxHeight = (c.maxh || 220) + 'px';   // scrollable group: long results stay inside a window of their own
  if (c.flow === 'wrap-left' && !host.dataset.flowed) { host.dataset.flowed = '1'; flowLeft(host); }
  return null;
};
const boxCls = (cls) => (c) => [cls, c.panel ? 'panel' : '', c.align === 'flex-end' ? 'end' : '', c.scroll ? 'scroll' : ''].filter(Boolean).join(' ');   // end: right-aligned column, the block grows leftward
const boxHit = (c) => !!(c.panel || c.scroll);

// A long column that must fit a short strip (the space under the active window): once attached, split it into the fewest columns whose
// height fits the strip. The first column sits at the RIGHT and the rest grow leftward, so it fills from the corner toward the free space.
function flowLeft(el) {
  let tries = 0;
  const go = () => {
    if (!el.isConnected) { if (tries++ < 20) setTimeout(go, 50); return; }
    const avail = (window.__view && window.__view.dock_h) || 0;
    const kids = [...el.children];
    if (avail < 90 || kids.length < 2) return;             // no usable strip: stay a plain column
    const gap = parseFloat(getComputedStyle(el).rowGap) * (window.__zoom || 1) || 6;
    const hs = kids.map((k) => k.getBoundingClientRect().height);
    const cols = [[]];
    let h = 0;
    kids.forEach((k, i) => {
      const cur = cols[cols.length - 1];
      if (cur.length && h + gap + hs[i] > avail) { cols.push([]); h = 0; }
      cols[cols.length - 1].push(k);
      h += (cols[cols.length - 1].length > 1 ? gap : 0) + hs[i];
    });
    if (cols.length < 2) return;                           // it already fits as one column
    el.classList.add('flow-left');
    el.replaceChildren(...cols.map((group) => { const c = document.createElement('div'); c.className = 'col end'; group.forEach((k) => c.append(k)); return c; }));
    window.dispatchEvent(new Event('morc-zoom'));
  };
  setTimeout(go, 120);                                     // after the first paint, so heights are real
}

// panel = a filled column for forms / grouped controls; column and row are hit boxes only when filled (panel) or scrolling
for (const [type, cls, panel] of [['column', 'col', false], ['row', 'row', false], ['panel', 'col', true]]) {
  register(type, (c, ctx) => {
    const spec = panel ? { ...c, panel: true } : c;
    const make = viewFactory(type + ':box', box(cls), { cls: boxCls(cls)(spec), hit: boxHit(spec), slot: true });
    return make(spec, ctx);
  });
}

registerView('spacer', ({ c, host }) => { host.style.height = (c.size ?? 6) + 'px'; return null; }, { cls: 'spacer', hit: false, tailwind: false });

registerView('divider', () => null, { cls: 'divider', tailwind: false });

// wing: the cohesion layout. `main` = the glanceable part, docked beside the active window (kept to the room that is free there);
// `aside` = the less useful part, tucked below and allowed to run wider, so the window's silhouette becomes an L instead of a slab.
// With morph low (or no neighbour) it is just a column. The window shape is the union of the [data-hit] plates, so gaps stay see-through.
registerView('wing', ({ c, ctx, host }) => {
  const v = window.__view || {};
  const morph = v.morph == null ? 0.6 : v.morph;
  const room = v.dock_w || 0;
  useLayoutEffect(() => {
    host.style.setProperty('--align', 'flex-start');
    const main = host.querySelector(':scope > .wing-main'), aside = host.querySelector(':scope > .wing-aside');
    main.append(...(c.main || []).map((id) => ctx.child(id)));
    aside.append(...(c.aside || []).map((id) => ctx.child(id)));
  }, []);
  const mw = morph >= 0.3 && room >= 200 ? Math.max(220, Math.min(room - 12, c.main_max || 330)) + 'px' : undefined;
  return html`<div class="col wing-main" style=${{ maxWidth: mw }}></div><div class="col wing-aside"></div>`;
}, { cls: 'col wing', hit: false });
