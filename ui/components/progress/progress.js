import { registerView, html, useEffect, useState } from '../../core/view.js';

// {type:"progress", label:"Video today", value:{bind:"/video"}, max:135, unit:"m", warnAt:.8, dangerAt:1}. Preact view.
registerView('progress', ({ c, useBind, host }) => {
  const label = useBind(c.label), v = useBind(c.value);
  const max = c.max || 100, n = Number(v) || 0, ratio = n / max, u = c.unit || '';
  host.className = 'c-prog ' + (ratio >= (c.dangerAt ?? 1) ? 'bad' : ratio >= (c.warnAt ?? 0.8) ? 'warn' : c.goodBelow != null && ratio < c.goodBelow ? 'good' : 'ok');
  // the bar springs up from zero on the frame after insert, then follows every change
  const [w, setW] = useState(0.02);
  useEffect(() => { requestAnimationFrame(() => requestAnimationFrame(() => setW(Math.max(0.02, Math.min(1, ratio))))); }, [ratio]);
  return html`
    <div class="top flex justify-between gap-3"><span class="lbl min-w-0 truncate">${label ?? ''}</span><span class="val shrink-0">${Number.isInteger(n) ? n : n.toFixed(1)}${u} / ${max}${u}</span></div>
    <div class="track"><div class="fill" style=${{ transform: `scaleX(${w})` }}></div></div>`;
}, { cls: 'c-prog' });
