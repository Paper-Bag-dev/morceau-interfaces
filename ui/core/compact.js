// Key + detail (the mentality, docs/theme-rules.md): compact shows a card's KEY part, hovering reveals the DETAIL, growing the way the card
// is long: tall parts (lists, asks, steps) fold downward, wide parts (.fold-x with --key-w: a pie's legend) fold to the right.
// Compact <-> full: what happens INSIDE a card when it opens or folds (core/bridge.js owns the zoom and the window; this owns the content).
// Every timing and size is data: limits reveal-ms (how long lists slide), tokens compact-rows / reveal-stagger-ms (tokens/plate.json).
// Opening and folding are mirrors of each other, so neither one ends in a jump:
//   expand():   lists slide open from their compact height to their full one, the tucked rows scale and fade in top to bottom, the
//               width eases out (titles reveal more characters), the edge thins and the dark layer fades (CSS transitions on #stage.compact).
//   collapse(): the same in reverse, started together with the shrink spring; the window only shrinks after both have settled.
import { enter } from './motion.js';

const LISTS = '.c-list, .c-pastes, .c-check, .c-asks';
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

function pinHeights(stage, ms) {
  // a CSS size transition needs a number at both ends: pin each fold at its full size while it moves (lists: height, .fold-x: width)
  for (const el of stage.querySelectorAll(LISTS)) { el.style.setProperty('--full-h', el.scrollHeight + 'px'); el.classList.add('revealing'); }
  for (const el of stage.querySelectorAll('.fold-x[data-key]')) { el.style.setProperty('--full-w', el.scrollWidth + 'px'); el.classList.add('revealing'); }
  setTimeout(() => stage.querySelectorAll(LISTS + ', .fold-x').forEach((el) => { el.style.removeProperty('--full-h'); el.style.removeProperty('--full-w'); el.classList.remove('revealing'); }), ms + 80);
}

/** Does this card have anything to fold (a key part with detail beside or below it)? */
export function folds(stage) { return !!stage.querySelector('.fold-x[data-key]'); }

const refit = () => window.dispatchEvent(new Event('morc-zoom'));   // components refit their key / detail sizes (a chart's canvas width)

/** Open: show everything that was tucked away. */
export function expand(stage, ms) {
  // sideways folds: draw the detail first (full canvas, still clipped to the key), then ease the clip open
  stage.dataset.opening = '1';
  refit();
  setTimeout(() => delete stage.dataset.opening, ms + 100);
  pinHeights(stage, ms);
  if (!reduced) {
    const cs = getComputedStyle(stage), shown = parseInt(cs.getPropertyValue('--compact-rows')) || 2, step = parseFloat(cs.getPropertyValue('--reveal-stagger-ms')) || 55;
    stage.querySelectorAll('.c-list').forEach((l) => [...l.children].slice(shown).forEach((row, i) => enter(row, 60 + i * step)));
    stage.querySelectorAll('.c-asks, .c-pastes, .c-check').forEach((l) => [...l.querySelectorAll('.ask, .paste, .row')].slice(1).forEach((row, i) => enter(row, 60 + i * step)));
  }
  stage.classList.remove('compact');
}

/** Fold: tuck the extra rows away again. Starts the CSS transitions now; call it together with the shrink spring. */
export function collapse(stage, ms) {
  pinHeights(stage, ms);
  // the transition needs one frame at the pinned full size before the compact size applies; once it has eased shut, components drop the
  // hidden detail (a chart's canvas shrinks to its key part) so the card can hug what is left
  requestAnimationFrame(() => stage.classList.add('compact'));
  setTimeout(refit, ms + 60);
}

/** Snap straight to compact or full, no motion (first show, a new layout from the host). */
export function set(stage, on) {
  if (stage.classList.contains('compact') === on) return;
  stage.classList.toggle('compact', on);
  refit();
}
