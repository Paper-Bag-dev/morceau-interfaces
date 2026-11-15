import { registerView, html } from '../../core/view.js';

// {type:"beatring", beat:{bind:"@/beat"}, media:{bind:"@/media"}}
// A circle visualizer: 64 radial bars around a ring, lengths from the 8 bands of @/beat (interpolated, mirrored), hue cycling
// round the ring (RGB); the core pulses on each beat and shows what is playing (@/media from the video card). Idle = a calm ring.
const N = 64, R = 62, MAX = 46, C = 120;
const band = (bands, i) => {
  const f = (i / N) * 2, x = (f > 1 ? 2 - f : f) * (bands.length - 1), a = Math.floor(x), t = x - a;
  return (bands[a] || 0) * (1 - t) + (bands[a + 1] ?? bands[a] ?? 0) * t;
};

registerView('beatring', ({ c, useBind, host }) => {
  const b = useBind(c.beat) || {}, media = useBind(c.media) || {};
  const on = !!b.on, bands = b.bands || [], hue = b.h || 200, p = b.p || 0, l = b.l || 0;
  host.title = media.title || 'Visualizer';
  const bars = Array.from({ length: N }, (_, i) => {
    const a = (i / N) * Math.PI * 2 - Math.PI / 2, v = on ? band(bands, i) : 0.06 + 0.04 * Math.sin(i * 0.7);
    const r0 = R + p * 6, r1 = r0 + 4 + v * MAX;
    return html`<line x1=${C + Math.cos(a) * r0} y1=${C + Math.sin(a) * r0} x2=${C + Math.cos(a) * r1} y2=${C + Math.sin(a) * r1}
      stroke=${`hsl(${(hue + (i / N) * 360) % 360} 100% ${on ? 62 : 45}%)`} stroke-width="4.2" stroke-linecap="round" opacity=${on ? 0.95 : 0.5} />`;
  });
  const state = media.state === 'playing' ? 'Playing' : media.state === 'paused' ? 'Paused' : media.state === 'ended' ? 'Ended' : on ? (b.demo ? 'Demo beat' : 'Live audio') : 'Waiting';
  return html`
    <svg class="br-svg" viewBox="0 0 240 240" role="img" aria-label=${`Visualizer, ${state}`}
        style=${`transform: rotate(${(hue * 0.4) % 360}deg) scale(${1 + p * 0.06})`}>
      <circle cx=${C} cy=${C} r=${R - 6 + p * 8} fill=${`hsl(${hue} 90% 55% / ${0.12 + l * 0.35})`} />
      <circle cx=${C} cy=${C} r=${R - 2 + p * 6} fill="none" stroke=${`hsl(${(hue + 180) % 360} 100% 70% / .7)`} stroke-width="1.5" />
      ${bars}
    </svg>
    <div class="br-mid"><span class="br-title">${media.title || 'Nothing playing'}</span><span class="br-state">${state}</span></div>`;
}, { cls: 'c-beatring' });
