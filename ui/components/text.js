import { register } from '../core/registry.js';

register('text', (c, ctx) => {
  const el = document.createElement('div');
  el.className = ['c-text', c.size || '', c.bold ? 'b' : '', c.tone ? 'tone-' + c.tone : ''].join(' ').trim();
  el.dataset.hit = '';
  ctx.bind(c.text, (v) => { el.textContent = v ?? ''; });
  if (c.tone && typeof c.tone === 'object') ctx.bind(c.tone, (t) => { el.className = el.className.replace(/tone-\w+/g, '') + (t ? ' tone-' + t : ''); });
  return el;
});

register('badge', (c, ctx) => {
  const el = document.createElement('span');
  el.className = 'badge' + (c.tone ? ' tone-' + c.tone : '');
  el.dataset.hit = '';
  ctx.bind(c.text, (v) => { el.textContent = v ?? ''; });
  return el;
});

register('kv', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'c-kv';
  el.dataset.hit = '';
  const k = document.createElement('span'), v = document.createElement('span');
  k.className = 'k';
  ctx.bind(c.k, (x) => { k.textContent = x ?? ''; });
  ctx.bind(c.v, (x) => { v.textContent = x ?? ''; });
  el.append(k, v);
  return el;
});
