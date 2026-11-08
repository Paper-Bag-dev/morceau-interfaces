import { register } from '../core/registry.js';
import { reduced } from '../core/motion.js';

// {type:"stat", label:"Video time today", value:{bind:"/video"}, unit:"min", max:135, warnAt:.8, dangerAt:1, decimals:0}
// A hero numeral that counts to its value, plus (with `max`) a hairline needle that glides to where it sits against the limit.
register('stat', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'stat';
  el.dataset.hit = '';
  el.innerHTML = '<div class="row1"><span class="num">0</span><span class="unit"></span></div><div class="lbl"></div>' +
    (c.max ? '<div class="needle"><div class="rail"></div><div class="fill"></div><div class="dot"></div></div>' : '');
  const num = el.querySelector('.num'), lbl = el.querySelector('.lbl'), fill = el.querySelector('.fill'), dot = el.querySelector('.dot');
  const unit = el.querySelector('.unit');
  ctx.bind(c.unit, (v) => { unit.textContent = v ?? ''; });   // the unit may be bound or templated, like label and value
  // decimals: as given, else as many as the value itself carries (2.5 stays 2.5, never rounds to 3), up to 2
  let dec = c.decimals ?? 0;
  const show = (n) => (dec ? n.toFixed(dec) : String(Math.round(n)));
  ctx.bind(c.label, (v) => { lbl.textContent = v ?? ''; });

  let shown = 0, raf = 0;
  ctx.bind(c.value, (v) => {
    const to = Number(v) || 0, from = shown, t0 = performance.now();
    if (c.decimals === undefined) dec = Math.min(2, (String(v).split('.')[1] || '').replace(/\D.*$/, '').length);
    cancelAnimationFrame(raf);
    const step = (now) => {                                  // count to the value: it answers a change, not decoration
      const p = Math.min(1, (now - t0) / 650), e = 1 - Math.pow(1 - p, 3);
      shown = from + (to - from) * e;
      num.textContent = show(shown);
      if (p < 1) raf = requestAnimationFrame(step);
    };
    if (reduced) { shown = to; num.textContent = show(to); } else raf = requestAnimationFrame(step);
    if (c.max) {
      const r = Math.max(0, Math.min(1, to / c.max));
      el.classList.remove('warn', 'bad');
      if (to / c.max >= (c.dangerAt ?? 1)) el.classList.add('bad'); else if (to / c.max >= (c.warnAt ?? 0.8)) el.classList.add('warn');
      requestAnimationFrame(() => requestAnimationFrame(() => { fill.style.transform = `scaleX(${r})`; dot.style.left = `${r * 100}%`; }));
    }
  });
  return el;
});
