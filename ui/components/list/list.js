import { register } from '../../core/registry.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"list", items:{bind:"/todos"}, max:6}   item: {text, sub?, badge?, action?}
register('list', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'c-list';
  let first = true;
  ctx.bind(c.items, (items) => {
    el.replaceChildren();
    (Array.isArray(items) ? items : []).slice(0, c.max || 8).forEach((it, i) => {
      const row = document.createElement('div');
      row.className = 'item' + (it.action ? ' act' : '');
      row.dataset.hit = '';
      const left = document.createElement('div');
      left.textContent = it.text ?? '';
      if (it.sub) { const s = document.createElement('div'); s.className = 'sub'; s.textContent = it.sub; left.append(s); }
      row.append(left);
      if (it.badge) { const b = document.createElement('span'); b.className = 'badge'; b.textContent = it.badge; row.append(b); }
      if (it.action) row.addEventListener('click', (e) => { rippleAt(row, e); ctx.act(it.action); });
      el.append(row);
      if (!first && !reduced) enter(row, i * 30);
    });
    first = false;
  });
  return el;
});
