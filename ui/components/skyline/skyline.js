import { registerView, html, useEffect, useState } from '../../core/view.js';
import { palette } from '../../charts/util.js';

// {type:"skyline", title:"{/title}", day:{bind:"/day"}, cats:[["study","Study"],...], beat:{bind:"@/beat"}}
// With music (`morc beat` feeds @/beat) each building jumps with its band and the lit windows cycle through RGB.
// The day as a 2D city: one building per hour since the day started, as tall as the minutes in it (an hour = full height).
// Floors are lit by category (study, video, games), stacked from the ground; quiet hours are dark low houses. A sun (day)
// or moon (after 19:00) hangs over the current hour. Buildings rise in, staggered, on mount and grow live as the file updates.
// day = {hours:[{h, <cat>: minutes, ...}], now_h} from any per-hour source (a JSON file bound with `sources`).
const CATS0 = [['study', 'Study'], ['video', 'Video'], ['games', 'Games']];   // default; `cats: [[key, label], ...]` sets your own
const W = 24, GAP = 6, H = 150, BASE = 172, FLOOR = 6;

registerView('skyline', ({ c, useBind, host }) => {
  const day = useBind(c.day) || {};
  const title = useBind(c.title) || 'Today as a city';
  const hours = day.hours || [];
  const CATS = useBind(c.cats) || CATS0;
  const beat = useBind(c.beat) || {};
  const on = !!beat.on, bands = beat.bands || [];
  const [up, setUp] = useState(false);
  const [settled, setSettled] = useState(false);
  const [tip, setTip] = useState(null);
  useEffect(() => { const t = setTimeout(() => setUp(true), 60), u = setTimeout(() => setSettled(true), 2600); return () => { clearTimeout(t); clearTimeout(u); }; }, []);
  const col = CATS.map((_, i) => palette(i, CATS.length));
  const tot = (r) => CATS.reduce((a, [k]) => a + (r[k] || 0), 0);
  const total = hours.reduce((a, r) => a + tot(r), 0);
  const width = Math.max(1, hours.length) * (W + GAP) + GAP;
  const nowI = day.now_h ?? hours.length - 1;
  const nowH = hours[nowI] ? hours[nowI].h : 0;
  const night = nowH >= 19 || nowH < 6;
  host.title = title;

  const buildings = hours.map((r, i) => {
    const x = GAP + i * (W + GAP);
    const t = Math.min(60, tot(r));
    const hgt = t ? Math.max(14, (t / 60) * H) : 10 + ((r.h * 7) % 9);
    let y = BASE, floors = [];
    CATS.forEach(([k], ci) => {
      const m = Math.min(60, r[k] || 0); if (!m) return;
      const fh = (m / Math.max(t, 1)) * hgt; y -= fh;
      floors.push(html`<rect x=${x} y=${y} width=${W} height=${fh} fill=${col[ci]} opacity=".9" />`);
      for (let wy = y + 4; wy < y + fh - 3; wy += FLOOR + 2)
        for (const wx of [x + 4, x + 10, x + 16])
          floors.push(html`<rect x=${wx} y=${wy} width="4" height=${FLOOR - 2} rx="1" fill=${on ? `hsl(${((beat.h || 0) + i * 18 + wy * 2) % 360} 100% 70%)` : '#fff'} opacity=${((i * 13 + wy * 7 + wx) % 5) ? (on ? 0.6 + (bands[i % 8] || 0) * 0.4 : 0.55) : 0.12} />`);
    });
    const dark = !t ? html`<rect x=${x + 2} y=${BASE - hgt} width=${W - 4} height=${hgt} rx="2" fill="rgb(255 255 255 / .08)" />` : null;
    const roof = t ? html`<rect x=${x} y=${BASE - hgt - 3} width=${W} height="3" rx="1.5" fill=${col[CATS.findIndex(([k]) => (r[k] || 0) === Math.max(...CATS.map(([kk]) => r[kk] || 0)))]} />` : null;
    return html`<g class="sk-b" style=${`--d:${i * 45}ms;--j:${on ? 1 + (bands[i % 8] || 0) * 0.28 + (beat.p || 0) * 0.08 : 1}`} data-hit
        onMouseEnter=${() => setTip({ i, x: x + W / 2, r, t })} onMouseLeave=${() => setTip(null)}>
      <rect x=${x - GAP / 2} y="0" width=${W + GAP} height=${BASE} fill="transparent" />
      ${dark}${floors}${roof}
      ${r.h % 3 === 0 ? html`<text x=${x + W / 2} y=${BASE + 16} class="sk-lab">${String(r.h).padStart(2, '0')}</text>` : null}
    </g>`;
  });

  const cx = GAP + nowI * (W + GAP) + W / 2;
  const orb = night
    ? html`<g class="sk-orb"><circle cx=${cx} cy="18" r="9" fill="#e9ecff" /><circle cx=${cx + 4} cy="15" r="8" fill="var(--sk-sky, #0e1020)" /></g>`
    : html`<g class="sk-orb"><circle cx=${cx} cy="18" r="13" fill="#ffcc4d" opacity=".18" /><circle cx=${cx} cy="18" r=${8 + (on ? (beat.p || 0) * 4 : 0)} fill=${on ? `hsl(${beat.h || 45} 100% 65%)` : '#ffcc4d'} /></g>`;

  return html`
    <div class="sk-head">
      <span class="sk-title">${title}</span>
      <span class="sk-hero">${total}<small> min</small></span>
    </div>
    <div class="sk-scene">
      <svg viewBox=${`0 0 ${width} ${BASE + 22}`} class=${'sk-svg' + (up ? ' up' : '') + (settled ? ' settled' : '') + (on ? ' dance' : '')} role="img" aria-label=${`${total} minutes today by hour`}>
        ${orb}
        <line x1="0" x2=${width} y1=${BASE + 0.5} y2=${BASE + 0.5} class="sk-ground" />
        ${buildings}
      </svg>
      ${tip ? html`<div class="sk-tip" style=${`left:${(tip.x / width) * 100}%`}>
        <b>${String(tip.r.h).padStart(2, '0')}:00</b> ${tip.t ? CATS.filter(([k]) => tip.r[k]).map(([k, l]) => `${l} ${tip.r[k]} min`).join(' · ') : 'Quiet hour'}
      </div>` : null}
    </div>
    <div class="sk-legend">${CATS.map(([, l], i) => html`<span><i style=${`background:${col[i]}`}></i>${l}</span>`)}</div>`;
}, { cls: 'c-skyline' });
