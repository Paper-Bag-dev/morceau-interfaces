import { register } from '../../core/registry.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"list", items:{bind:"/todos"}, max:6}   item: {text, sub?, badge?, tip?, action?}   sizes: ui/tokens/list.json
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
      left.className = 'txt';
      const ttl = document.createElement('div'); ttl.className = 'ttl'; ttl.textContent = it.text ?? ''; left.append(ttl);
      if (it.sub) { const s = document.createElement('div'); s.className = 'sub'; s.textContent = it.sub; left.append(s); }
      // each line is cut to one line with an ellipsis, so the full text is always in the tooltip (item.tip overrides it)
      row.title = it.tip ?? [it.text, it.sub].filter(Boolean).join('\n');
      row.append(left);
      if (it.badge) { const b = document.createElement('span'); b.className = 'badge'; b.textContent = it.badge; row.append(b); }
      if (it.action) row.addEventListener('click', (e) => { rippleAt(row, e); ctx.act(it.action); });
      el.append(row);
      if (!reduced) enter(row, (first ? 60 : 0) + i * 35);   // rows fade in one after another, also on first show
    });
    first = false;
  });
  return el;
});
