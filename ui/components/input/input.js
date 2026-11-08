import { register } from '../../core/registry.js';

// {type:"input", placeholder:"Why do you need it?", bind:"/draft", submit:{do:"wake", payload:{answer:"{/draft}"}}}
// Focusing a field asks the host for real keyboard focus (windows are otherwise no-activate); blur gives it back.
register('input', (c, ctx) => {
  const el = document.createElement('input');
  el.className = 'field';
  el.type = 'text';
  el.dataset.hit = '';
  el.dataset.nodrag = '';
  el.placeholder = c.placeholder || '';
  el.spellcheck = false;
  ctx.bind({ bind: c.bind }, (v) => { if (document.activeElement !== el) el.value = v ?? ''; });
  el.addEventListener('input', () => ctx.store.set(c.bind, el.value));
  el.addEventListener('focus', () => ctx.send({ t: 'kbd', v: true }));
  el.addEventListener('blur', () => ctx.send({ t: 'kbd', v: false }));
  el.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && c.submit) { ctx.act(c.submit); if (c.clear) { ctx.store.set(c.bind, ''); el.value = ''; } el.blur(); }
    if (e.key === 'Escape') el.blur();
  });
  return el;
});
