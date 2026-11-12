import { registerView, html, useEffect, useState } from '../../core/view.js';
import { reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"checklist", title:"To do today", items:{bind:"/items"}, run:"tick-", max:12}
// item: {id, text, sub?, badge?, done}. A click flips item.done in the model at once (the row answers instantly), then asks the host to run the
// allowlisted action <run><id> (see actions.json), which stores the new state as JSON. The page re-reads that JSON through `sources`, so the file is the truth.
// An open item whose badge is a clock time (HH:MM) that has already passed gets a coral ring: "this one is due". Preact view.
const dueNow = (badge) => {
  const m = /^(\d{1,2}):(\d{2})$/.exec(badge || '');
  if (!m) return false;
  const n = new Date();
  return n.getHours() * 60 + n.getMinutes() >= Number(m[1]) * 60 + Number(m[2]);
};

registerView('checklist', ({ c, ctx, useBind, act }) => {
  const title = useBind(c.title);
  const raw = useBind(c.items);
  const items = (Array.isArray(raw) ? raw : []).slice(0, c.max || 12);
  const path = c.items && c.items.bind;
  const [, tick] = useState(0);
  useEffect(() => { if (reduced) return; const t = setInterval(() => tick((n) => n + 1), 60000); return () => clearInterval(t); }, []);   // "due" rings follow the clock
  const done = items.filter((i) => i.done).length;
  const [w, setW] = useState(0.02);
  useEffect(() => { requestAnimationFrame(() => setW(items.length ? Math.max(0.02, done / items.length) : 0.02)); }, [done, items.length]);
  const flip = (i) => (e) => {
    if (e && e.type === 'click') rippleAt(e.currentTarget, e);
    const cur = (ctx.store.get(path) || [])[i];
    if (!cur) return;
    ctx.store.set(`${path}/${i}/done`, !cur.done);                      // instant answer on the page
    act({ do: 'run', id: `${c.run || 'tick-'}${cur.id}`, into: '/_tick' });   // the host stores it as JSON (and ticks Todoist)
  };
  const key = (i) => (e) => { if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); flip(i)(e); } };
  return html`
    <div class="head flex items-baseline justify-between gap-3"><span class="ttl min-w-0 truncate">${title ?? ''}</span><span class="count shrink-0"><b class="n">${done}</b><span class="of"> of ${items.length}</span></span></div>
    <div class="track"><div class="fill" style=${{ transform: `scaleX(${w})` }}></div></div>
    <div class="rows" role="list">
      ${items.map((it, i) => html`<div key=${it.id ?? i} class=${'row' + (it.done ? ' done' : '') + (!it.done && dueNow(it.badge) ? ' due' : '')}
          data-hit="" role="checkbox" tabIndex="0" aria-checked=${it.done ? 'true' : 'false'} onClick=${flip(i)} onKeyDown=${key(i)}>
        <span class="box"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5.5 12.5l4.2 4.2L18.5 7.5" /></svg></span>
        <span class="txt min-w-0"><span class="t">${it.text ?? ''}</span><span class="sub">${it.sub ?? ''}</span></span>
        <span class="badge shrink-0">${it.badge ?? ''}</span>
      </div>`)}
    </div>`;
}, { cls: 'c-check plate' });
