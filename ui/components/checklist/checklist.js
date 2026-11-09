import { register } from '../../core/registry.js';
import { reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"checklist", title:"To do today", items:{bind:"/items"}, run:"tick-", max:12}
// item: {id, text, sub?, badge?, done}. A click flips item.done in the model at once (the row answers instantly), then asks the host to run the
// allowlisted action <run><id> (see actions.json), which stores the new state as JSON. The page re-reads that JSON through `sources`, so the file is the truth.
// An open item whose badge is a clock time (HH:MM) that has already passed gets a coral ring: "this one is due".
const CHECK = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5.5 12.5l4.2 4.2L18.5 7.5" /></svg>';

const dueNow = (badge) => {
  const m = /^(\d{1,2}):(\d{2})$/.exec(badge || '');
  if (!m) return false;
  const n = new Date();
  return n.getHours() * 60 + n.getMinutes() >= Number(m[1]) * 60 + Number(m[2]);
};

register('checklist', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'c-check plate';
  el.dataset.hit = '';
  el.innerHTML = '<div class="head"><span class="ttl"></span><span class="count"><b class="n">0</b><span class="of"></span></span></div>'
    + '<div class="track"><div class="fill"></div></div><div class="rows" role="list"></div>';
  const ttl = el.querySelector('.ttl'), n = el.querySelector('.n'), of = el.querySelector('.of');
  const fill = el.querySelector('.fill'), rowsEl = el.querySelector('.rows');
  const path = c.items && c.items.bind;
  let built = [];

  ctx.bind(c.title, (v) => { ttl.textContent = v ?? ''; });

  const paint = (items) => {
    const done = items.filter((i) => i.done).length;
    n.textContent = String(done);
    of.textContent = ` of ${items.length}`;
    requestAnimationFrame(() => { fill.style.transform = `scaleX(${items.length ? Math.max(0.02, done / items.length) : 0.02})`; });
    built.forEach((r, i) => {
      const it = items[i];
      r.classList.toggle('done', !!it.done);
      r.classList.toggle('due', !it.done && dueNow(it.badge));
      r.setAttribute('aria-checked', it.done ? 'true' : 'false');
    });
  };

  const build = (items) => {
    rowsEl.replaceChildren();
    built = items.slice(0, c.max || 12).map((it, i) => {
      const row = document.createElement('div');
      row.className = 'row';
      row.dataset.hit = '';
      row.setAttribute('role', 'checkbox');
      row.tabIndex = 0;
      row.innerHTML = `<span class="box">${CHECK}</span><span class="txt"><span class="t"></span><span class="sub"></span></span><span class="badge"></span>`;
      row.querySelector('.t').textContent = it.text ?? '';
      row.querySelector('.sub').textContent = it.sub ?? '';
      row.querySelector('.badge').textContent = it.badge ?? '';
      const flip = (e) => {
        if (e) rippleAt(row, e);
        const cur = (ctx.store.get(path) || [])[i];
        if (!cur) return;
        ctx.store.set(`${path}/${i}/done`, !cur.done);                      // instant answer on the page
        ctx.act({ do: 'run', id: `${c.run || 'tick-'}${cur.id}`, into: '/_tick' });   // the host stores it as JSON (and ticks Todoist)
      };
      row.addEventListener('click', flip);
      row.addEventListener('keydown', (e) => { if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); flip(); } });
      rowsEl.append(row);
      return row;
    });
  };

  ctx.bind(c.items, (v) => {
    const items = Array.isArray(v) ? v : [];
    const same = built.length === Math.min(items.length, c.max || 12) && built.every((r, i) => r.querySelector('.t').textContent === (items[i].text ?? ''));
    if (!same) build(items);                                                  // rebuild only when the list itself changed, so a tick animates in place
    paint(items.slice(0, c.max || 12));
  });
  if (!reduced) setInterval(() => ctx.store.get(path) && paint((ctx.store.get(path) || []).slice(0, c.max || 12)), 60000);   // "due" rings follow the clock
  return el;
});
