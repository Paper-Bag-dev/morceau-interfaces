// Click ripple from the exact pointer position. Shared by buttons and clickable list rows.
import { reduced } from './motion.js';

export function rippleAt(el, e) {
  if (reduced) return;
  const z = window.__zoom || 1;                       // rects/pointer are in zoomed px, styles are not
  const r = el.getBoundingClientRect();
  const d = (Math.max(r.width, r.height) / z) * 2.2;
  const s = document.createElement('span');
  s.className = 'ripple';
  s.style.cssText = `width:${d}px;height:${d}px;left:${(e.clientX - r.left) / z - d / 2}px;top:${(e.clientY - r.top) / z - d / 2}px`;
  el.append(s);
  s.animate([{ transform: 'scale(0)', opacity: 0.55 }, { transform: 'scale(1)', opacity: 0 }], { duration: 560, easing: 'cubic-bezier(.16,1,.3,1)' }).finished.then(() => s.remove(), () => s.remove());
}
