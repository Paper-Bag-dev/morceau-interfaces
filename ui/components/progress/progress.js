import { register } from '../../core/registry.js';

// {type:"progress", label:"Video today", value:{bind:"/video"}, max:135, unit:"m", warnAt:.8, dangerAt:1}
register('progress', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'c-prog';
  el.dataset.hit = '';
  el.innerHTML = '<div class="top"><span class="lbl"></span><span class="val"></span></div><div class="track"><div class="fill"></div></div>';
  const lbl = el.querySelector('.lbl'), val = el.querySelector('.val'), fill = el.querySelector('.fill');
  const max = c.max || 100;
  ctx.bind(c.label, (v) => { lbl.textContent = v ?? ''; });
  ctx.bind(c.value, (v) => {
    const n = Number(v) || 0, ratio = n / max;
    val.textContent = `${Number.isInteger(n) ? n : n.toFixed(1)}${c.unit || ''} / ${max}${c.unit || ''}`;
    el.classList.remove('warn', 'bad', 'good');
    el.classList.add(ratio >= (c.dangerAt ?? 1) ? 'bad' : ratio >= (c.warnAt ?? 0.8) ? 'warn' : c.goodBelow != null && ratio < c.goodBelow ? 'good' : 'ok');
    // transition on the first frame after insert so the bar springs up from zero
    requestAnimationFrame(() => requestAnimationFrame(() => { fill.style.transform = `scaleX(${Math.max(0.02, Math.min(1, ratio))})`; }));
  });
  return el;
});
