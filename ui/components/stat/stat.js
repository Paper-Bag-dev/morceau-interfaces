import { registerView, html, useEffect, useRef, useState } from '../../core/view.js';
import { reduced } from '../../core/motion.js';

// {type:"stat", label:"Video time today", value:{bind:"/video"}, unit:"min", max:135, warnAt:.8, dangerAt:1, decimals:0}
// A hero numeral that counts to its value, plus (with `max`) a hairline needle that glides to where it sits against the limit. Preact view.
registerView('stat', ({ c, useBind, host }) => {
  const v = useBind(c.value), unit = useBind(c.unit), label = useBind(c.label);
  const to = Number(v) || 0;
  // decimals: as given, else as many as the value itself carries (2.5 stays 2.5, never rounds to 3), up to 2
  const dec = c.decimals ?? Math.min(2, (String(v ?? '').split('.')[1] || '').replace(/\D.*$/, '').length);
  const [shown, setShown] = useState(reduced ? to : 0);
  const from = useRef(shown);
  useEffect(() => {                                     // count to the value: it answers a change, not decoration
    if (reduced) { setShown(to); return; }
    const a = from.current, t0 = performance.now();
    let raf = 0;
    const step = (now) => {
      const p = Math.min(1, (now - t0) / 650), e = 1 - Math.pow(1 - p, 3);
      const x = a + (to - a) * e;
      from.current = x; setShown(x);
      if (p < 1) raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  }, [to]);
  const r = c.max ? Math.max(0, Math.min(1, to / c.max)) : 0;
  const [r2, setR2] = useState(0);
  useEffect(() => { requestAnimationFrame(() => requestAnimationFrame(() => setR2(r))); }, [r]);   // the needle glides in from zero
  const state = c.max ? (to / c.max >= (c.dangerAt ?? 1) ? 'bad' : to / c.max >= (c.warnAt ?? 0.8) ? 'warn' : '') : '';
  host.className = ['stat', state].filter(Boolean).join(' ');
  return html`
    <div class="row1 flex items-baseline gap-1"><span class="num">${dec ? shown.toFixed(dec) : String(Math.round(shown))}</span><span class="unit">${unit ?? ''}</span></div>
    <div class="lbl">${label ?? ''}</div>
    ${c.max ? html`<div class="needle"><div class="rail"></div><div class="fill" style=${{ transform: `scaleX(${r2})` }}></div><div class="dot" style=${{ left: `${r2 * 100}%` }}></div></div>` : null}`;
}, { cls: 'stat' });
