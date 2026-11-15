// beat: every card dances to the shared /beat value (fed by `morc beat`): plates bounce and sway on each beat, an RGB glow
// cycles its hue with the music. Idle (no feed, or /beat/on false) it does nothing. Repo plugin (loads in every card).
// /beat = {on, l: level 0..1, p: beat pulse 0..1, h: hue 0..360, sw: -1|1 sway side, bands: [8 x 0..1]}
import { shared } from '../core/sharedbind.js';

const root = document.documentElement;
const seed = [...(location.search + location.hash)].reduce((a, ch) => (a * 31 + ch.charCodeAt(0)) >>> 0, 7) % 360; // each card its own phase
const css = document.createElement('style');
css.textContent = `
html.beat-on .card-plate { transition: transform 90ms ease-out, box-shadow 90ms ease-out, outline-color 90ms;
  transform: translateY(calc(var(--bt-p, 0) * -7px)) rotate(calc(var(--bt-sw, 0) * var(--bt-p, 0) * 1.4deg)) scale(calc(1 + var(--bt-p, 0) * .03));
  box-shadow: inset 0 0 calc(10px + var(--bt-l, 0) * 40px) hsl(var(--bt-h, 0) 100% 60% / calc(.25 + var(--bt-l, 0) * .6));
  outline: 1.5px solid hsl(var(--bt-h, 0) 100% 65% / calc(.25 + var(--bt-l, 0) * .7)); outline-offset: -1.5px; }
@media (prefers-reduced-motion: reduce) { html.beat-on .card-plate { transform: none; } }`;
document.head.appendChild(css);

const s = shared();
s.sub('/beat', (b) => {
  if (!b || !b.on) { root.classList.remove('beat-on'); return; }
  root.classList.add('beat-on');
  root.style.setProperty('--bt-l', (+b.l || 0).toFixed(3));
  root.style.setProperty('--bt-p', (+b.p || 0).toFixed(3));
  root.style.setProperty('--bt-h', String(((+b.h || 0) + seed) % 360));
  root.style.setProperty('--bt-sw', String(b.sw || 1));
});
