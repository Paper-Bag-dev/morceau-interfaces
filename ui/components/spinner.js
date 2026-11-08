import { register } from '../core/registry.js';

// {type:"spinner", label:"Reading your last 7 days"}: shown while something is being built; pair it with `when` on a stage value
// so it disappears the moment the result arrives. The ring is the only always-moving thing on screen, so it means "working".
register('spinner', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'spinner plate';
  el.dataset.hit = '';
  el.innerHTML = '<span class="ring"></span><span class="lb"></span>';
  ctx.bind(c.label, (v) => { el.querySelector('.lb').textContent = v ?? ''; });
  return el;
});