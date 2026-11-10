// Compact <-> full: what happens INSIDE a card when it opens or folds (core/bridge.js owns the zoom and the window; this owns the content).
// Every timing and size is data: limits reveal-ms (how long lists slide), tokens compact-rows / reveal-stagger-ms (tokens/plate.json).
// Opening and folding are mirrors of each other, so neither one ends in a jump:
//   expand():   lists slide open from their compact height to their full one, the tucked rows scale and fade in top to bottom, the
//               width eases out (titles reveal more characters), the edge thins and the dark layer fades (CSS transitions on #stage.compact).
//   collapse(): the same in reverse, started together with the shrink spring; the window only shrinks after both have settled.
import { enter } from './motion.js';

const LISTS = '.c-list, .c-pastes, .c-check';
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

function pinHeights(stage, ms) {
  // a CSS height transition needs a number at both ends: pin each list at its current full height while it moves
  for (const el of stage.querySelectorAll(LISTS)) { el.style.setProperty('--full-h', el.scrollHeight + 'px'); el.classList.add('revealing'); }
  setTimeout(() => stage.querySelectorAll(LISTS).forEach((el) => { el.style.removeProperty('--full-h'); el.classList.remove('revealing'); }), ms + 80);
}

/** Open: show everything that was tucked away. */
export function expand(stage, ms) {
  pinHeights(stage, ms);
  if (!reduced) {
    const cs = getComputedStyle(stage), shown = parseInt(cs.getPropertyValue('--compact-rows')) || 2, step = parseFloat(cs.getPropertyValue('--reveal-stagger-ms')) || 55;
    stage.querySelectorAll('.c-list').forEach((l) => [...l.children].slice(shown).forEach((row, i) => enter(row, 60 + i * step)));
  }
  stage.classList.remove('compact');
}

/** Fold: tuck the extra rows away again. Starts the CSS transitions now; call it together with the shrink spring. */
export function collapse(stage, ms) {
  pinHeights(stage, ms);
  // the transition needs one frame at the pinned full height before the compact height applies
  requestAnimationFrame(() => stage.classList.add('compact'));
}

/** Snap straight to compact or full, no motion (first show, a new layout from the host). */
export function set(stage, on) {
  stage.classList.toggle('compact', on);
}
