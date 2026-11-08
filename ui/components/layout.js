import { register } from '../core/registry.js';

const box = (cls) => (c, ctx) => {
  const el = document.createElement('div');
  el.className = cls + (c.panel ? ' panel' : '');
  if (c.gap != null) el.style.setProperty('--gap', c.gap + 'px');
  if (c.align) el.style.setProperty('--align', c.align);
  if (c.align === 'flex-end') el.classList.add('end');   // right-aligned column: items and wrapped text hug the right edge, the block grows leftward
  if (c.panel) el.dataset.hit = '';
  if (c.scroll) { el.classList.add('scroll'); el.style.maxHeight = (c.maxh || 220) + 'px'; el.dataset.hit = ''; }   // scrollable group: long results stay inside a window of their own
  ctx.children.forEach((make) => el.append(make()));
  if (c.flow === 'wrap-left') flowLeft(el);
  return el;
};

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

register('column', box('col'));
register('row', box('row'));
register('panel', (c, ctx) => box('col')({ ...c, panel: true }, ctx));   // filled card for forms / grouped controls

register('spacer', (c) => {
  const el = document.createElement('div');
  el.style.height = (c.size ?? 6) + 'px';
  return el;
});

register('divider', () => {
  const el = document.createElement('div');
  el.className = 'divider';
  el.dataset.hit = '';
  return el;
});

// wing: the cohesion layout. `main` = the glanceable part, docked beside the active window (kept to the room that is free there);
// `aside` = the less useful part, tucked below and allowed to run wider, so the window's silhouette becomes an L instead of a slab.
// With morph low (or no neighbour) it is just a column. The window shape is the union of the [data-hit] plates, so gaps stay see-through.
register('wing', (c, ctx) => {
  const v = window.__view || {};
  const morph = v.morph == null ? 0.6 : v.morph;
  const room = v.dock_w || 0;
  const el = document.createElement('div');
  el.className = 'col wing';
  el.style.setProperty('--align', 'flex-start');
  const mk = (ids) => ids.map((id) => ctx.child(id));
  const main = document.createElement('div');
  main.className = 'col wing-main';
  main.append(...mk(c.main || []));
  if (morph >= 0.3 && room >= 200) main.style.maxWidth = Math.max(220, Math.min(room - 12, c.main_max || 330)) + 'px';
  const aside = document.createElement('div');
  aside.className = 'col wing-aside';
  aside.append(...mk(c.aside || []));
  el.append(main, aside);
  return el;
});