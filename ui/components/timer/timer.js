import { registerView, html, useEffect, useState, Icon } from '../../core/view.js';

// {type:"timer", title:"{/title}", start:{bind:"/start"}, goal_min:{bind:"/goal_min"}, paused_at:{bind:"/paused_at"}, paused_ms:{bind:"/paused_ms"}}
// A small ring: the arc fills toward the goal, the elapsed time sits inside (seconds lighter, under it), the end time under the ring,
// one icon button to pause / resume. start / paused_at are epoch ms, paused_ms the total paused; the button writes them back with set
// actions, so the card data (and opts.persist) is the whole state. Past the goal the ring turns warn and keeps counting.
const pad = (n) => String(n).padStart(2, '0');
const clock = (ms) => new Date(ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
const R = 44, C = 2 * Math.PI * R;

registerView('timer', ({ c, useBind, act, host }) => {
  const title = useBind(c.title) || 'Timer';
  const start = Number(useBind(c.start)) || 0;
  const goal = Number(useBind(c.goal_min)) || 0;
  const pausedAt = Number(useBind(c.paused_at)) || 0;
  const pausedMs = Number(useBind(c.paused_ms)) || 0;
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    if (pausedAt) return undefined;
    const t = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(t);
  }, [pausedAt]);

  const el = Math.max(0, (pausedAt || now) - start - pausedMs);
  const s = Math.floor(el / 1000), hh = Math.floor(s / 3600), mm = Math.floor((s % 3600) / 60), ss = s % 60;
  const goalMs = goal * 60000, over = goalMs && el > goalMs;
  const ratio = goalMs ? (over ? 1 : el / goalMs) : (s % 60) / 60;
  const endAt = start + pausedMs + goalMs + (pausedAt ? now - pausedAt : 0);
  host.className = 'c-timer' + (pausedAt ? ' paused' : '') + (over ? ' over' : '');
  host.title = title;

  const toggle = () => {
    const t = Date.now();
    if (pausedAt) act([{ do: 'set', path: '/paused_ms', value: pausedMs + (t - pausedAt) }, { do: 'set', path: '/paused_at', value: 0 }]);
    else act({ do: 'set', path: '/paused_at', value: t });
  };
  const foot = !goal ? (pausedAt ? 'Paused' : title) : over ? `+${Math.floor((el - goalMs) / 60000)} min over` : pausedAt ? 'Paused' : `Ends ${clock(endAt)}`;

  return html`
    <div class="tm-ring">
      <svg viewBox="0 0 100 100" aria-hidden="true">
        <circle class="tm-track" cx="50" cy="50" r=${R} />
        <circle class="tm-arc" cx="50" cy="50" r=${R} stroke-dasharray=${C} stroke-dashoffset=${C * (1 - Math.max(0.004, ratio))} />
      </svg>
      <div class="tm-in" aria-label="Elapsed time"><span class="tm-hm">${hh}:${pad(mm)}</span><span class="tm-s">${pad(ss)} s</span></div>
    </div>
    <div class="tm-foot">
      <span class="tm-cap">${foot}</span>
      <button class="btn ghost icon-only tm-btn" data-hit title=${pausedAt ? 'Resume the timer' : 'Pause the timer'} onClick=${toggle}><${Icon} name=${pausedAt ? 'play' : 'pause'} size=${14} /></button>
    </div>`;
}, { cls: 'c-timer' });
