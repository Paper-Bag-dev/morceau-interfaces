// "The system is working": heavy components (3D model, graph, big charts) wear a slow blue pulse while they load; when ready the pulse stops
// and ONE radial ring goes out, then nothing. Blue means the system is busy; coral stays "yours" (drag glow, accent). Nothing loops after it is ready.
import { reduced } from './motion.js';

export function working(el) {
  el.classList.add('working');
  let finished = false;
  return {
    done() {
      if (finished) return;
      finished = true;
      el.classList.remove('working');
      if (reduced) return;
      const ring = document.createElement('span');
      ring.className = 'burst';
      el.append(ring);
      ring.animate([{ transform: 'scale(.55)', opacity: 0.7 }, { transform: 'scale(1.3)', opacity: 0 }], { duration: 800, easing: 'cubic-bezier(.16,1,.3,1)' })
        .finished.then(() => ring.remove(), () => ring.remove());
    },
  };
}