import * as compact from './compact.js';
// Page <-> host plumbing: messages, window shape, hover, hold-to-drag, pin, dismiss.

// Minimum / maximum sizes and timings are DATA: ui/limits.json, sent by the host (overridable in prefs/ui.json). The numbers in lim(...) calls are only fallbacks.
let limits = {};
export function setLimits(l) { limits = l || {}; }
const lim = (k, d) => (typeof limits[k] === 'number' ? limits[k] : d);
export const limit = lim;   // read a limit by name from other modules (main.js uses it for the attention pulse)

export const send = (m) => window.ipc && window.ipc.postMessage(JSON.stringify(m));

const stage = document.getElementById('stage');
const pin = document.getElementById('pin');

// ---------- window shape: report size + the rounded rects that stay clickable ----------
let lastSent = '', queued = false, restPending = false;

// ---------- scale: importance x size_bias, capped to 1/4 of the screen (host sends view.maxw/maxh) ----------
let view = { scale: 1, fill: false, maxw: 1e5, maxh: 1e5 }, zoom = 1, fits = 0;

function applyZoom(z) {
  zoom = z;
  window.__zoom = z;
  document.body.style.zoom = String(z);
  const surf = document.getElementById('surface');
  if (surf) surf.style.maxWidth = view.wrap > 0 ? Math.round(view.wrap / z - 52) + 'px' : '';   // long text wraps to the gap beside the neighbour
  window.dispatchEvent(new Event('morc-zoom'));   // charts repaint at the right resolution
}

export function setView(v) {
  view = { ...view, ...v };
  fits = 0;
  // one size per card, no hover scaling (user, 11-11): the chip's opts.size picks a factor from limits (base / medium / large-scale)
  const sizeK = stage.dataset.size === 'medium' ? lim('medium-scale', 1.12) : stage.dataset.size === 'large' ? lim('large-scale', 1.3) : lim('base-scale', 1);
  applyZoom(Math.max(lim('min-zoom', 0.85), (view.scale || 1) * sizeK));
  compact.set(stage, FOLDS() && !open);
  measureSoon();
}

// ---------- key + detail (docs/theme-rules.md 14): a card with more to show starts folded on its KEY part and hovering reveals the
// DETAIL; leaving folds it back. Content only: the zoom never changes (user, 11-11: no hover scaling), the window follows the content.
let open = false, openTimer = 0, shutTimer = 0;
const VISUAL = '.c-model, .c-graph, .c-chart canvas';
const REVEAL = () => lim('reveal-ms', 380);
// what folds: a chart with a key part (a pie's ring, its legend beside it); a list; any card taller than limits.compact-if-taller.
// Charts without a key part, graphs and 3D never fold; chips (opts.size) "medium" / "large" only fold when tall.
function FOLDS() {
  if (compact.folds(stage)) return true;
  if (stage.querySelector(VISUAL)) return false;
  const sz = stage.dataset.size;
  if (sz === 'large') return false;
  // only cards with rows to tuck away fold (a text-only card has nothing to reveal, so it never wears the fold edge): lists fold at any
  // size but large (a medium pill list starts on its first rows and grows on hover); pastes, checklists and asks fold when tall
  if (stage.querySelector('.c-list')) return true;
  // a long paragraph (more than limits.text-heavy-chars) folds to its first lines and opens on hover
  if ([...stage.querySelectorAll('.c-text:not(.lg)')].some((t) => t.textContent.length > lim('text-heavy-chars', 160))) return true;
  return !!stage.querySelector('.c-pastes, .c-check, .c-asks') && (sz === 'compact' || stage.dataset.tall === '1');
}
/** A card whose chip names no size and that is mostly text (more than limits.text-heavy-chars characters, no chart, graph or 3D)
 *  is a medium card by default (user, 11-11): bigger text you can read. Returns true when it changed the size. */
export function autoSize() {
  if (stage.dataset.size || stage.querySelector(VISUAL)) return false;
  const chars = (document.getElementById('surface')?.textContent || '').replace(/\s+/g, ' ').length;
  if (chars <= lim('text-heavy-chars', 160)) return false;
  stage.dataset.size = 'medium';
  return true;
}
stage.addEventListener('pointerenter', () => {
  clearTimeout(shutTimer);
  if (open || stage.classList.contains('dragging') || !stage.classList.contains('compact')) return;
  clearTimeout(openTimer);
  openTimer = setTimeout(() => { open = true; stage.classList.add('grown'); compact.expand(stage, REVEAL()); measureSoon(); }, lim('hover-delay-ms', 220));   // grown first: the charts redraw inside expand() and label every value
});
stage.addEventListener('pointerleave', () => {
  clearTimeout(openTimer);
  if (!open) return;
  clearTimeout(shutTimer);
  shutTimer = setTimeout(() => { open = false; compact.collapse(stage, REVEAL()); stage.classList.remove('grown'); measureSoon(); }, lim('shrink-delay-ms', 450));
});

let morphing = false, entered = false;   // entered: no native glass before the card's arrival starts (it flashed full size)
/** The native glass follows the card through an animation it did not start (the arrival): the plate's live rect every frame until `anim` ends. */
export function followGlass(anim) {
  entered = true;
  if (!anim) { measure(); return; }
  morphing = true;
  const tick = () => { if (anim.playState === 'finished' || anim.playState === 'idle') { morphing = false; measure(); return; } send({ t: 'glass', glass: glassRects(1) }); requestAnimationFrame(tick); };
  requestAnimationFrame(tick);
}

// each card plate's exact rounded rect: where the host may put native glass (macOS); the page draws the tint, grain and sheen on top.
function glassRects(f) {
  const sr = stage.getBoundingClientRect();
  return [...document.querySelectorAll('.card-plate')].map((el) => {
    const r = el.getBoundingClientRect();
    const br = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
    return [(r.left - sr.left) * f, (r.top - sr.top) * f, r.width * f, r.height * f, Math.min(br * zoom, Math.min(r.width, r.height) / 2) * f].map((n) => Math.round(n));
  }).filter((r) => r[2] > 0 && r[3] > 0);
}

// .hovered shows the pin and close (plate.css)
stage.addEventListener('pointerenter', () => stage.classList.add('hovered'));
stage.addEventListener('pointerleave', () => stage.classList.remove('hovered'));

function measure() {
  queued = false;
  // Never measure mid-animation: transforms (the pop-in, hover lifts) shift the rects, and the window shape would
  // lock to that moment. Wait for finite animations in the surface to end, then measure the resting layout.
  // (pseudo-element animations, like the folded edge's shimmer, never change the layout: they do not hold the window open)
  const running = document.getAnimations().filter((a) => a.playState === 'running' && a.effect && !a.effect.pseudoElement && a.effect.getTiming().iterations !== Infinity && stage.contains(a.effect.target));
  // Measure now regardless (a streamed build keeps starting new animations, so waiting for calm would freeze the window at its old size),
  // and measure again once everything is at rest; that second pass is what fixes the shape if this one caught a transform mid-flight.
  if (running.length && !restPending) {
    restPending = true;
    Promise.allSettled(running.map((a) => a.finished)).then(() => { restPending = false; setTimeout(measureSoon, 40); });
  }
  const sr = stage.getBoundingClientRect();
  // content can overflow its box (wide charts, long rows): measure the real extent, in zoomed px
  const w0 = Math.max(sr.width, stage.scrollWidth * zoom), h0 = Math.max(sr.height, stage.scrollHeight * zoom);
  const k = Math.min(view.maxw / w0, view.maxh / h0);
  // never while open on hover: that would rescale the text (no hover scaling); the window may pass the cap for the hover
  if (fits < 5 && !open && (k < 1 || (view.fill && k > 1.04 && zoom < lim('max-zoom', 2.2)))) {
    const next = Math.max(lim('min-zoom', 0.85), Math.min(lim('max-zoom', 2.2), zoom * k * 0.97));
    if (Math.abs(next - zoom) / zoom > 0.015) { fits++; applyZoom(next); return measureSoon(); }
  }
  let rects;
  // mid-animation (the arrival spring): the content's rects are transformed (the arrival starts at 8% size), and a window
  // shape cut to them clipped the whole spring away, so the card just popped in. Keep the whole window open until everything rests.
  if (running.length || stage.classList.contains('urgent') || stage.classList.contains('dragging')) {
    rects = [[3, 3, w0 - 6, h0 - 6, 28]];
  } else {
    rects = [...document.querySelectorAll('[data-hit]')].map((el) => {
      let r = el.getBoundingClientRect();
      // a child scrolled out of its scroll box must not leave an invisible clickable zone: clip to the scroller
      const sc = el.parentElement && el.parentElement.closest(stage.classList.contains('compact') ? '.scroll, .c-list' : '.scroll');
      if (sc) { const s = sc.getBoundingClientRect(); const l = Math.max(r.left, s.left), t = Math.max(r.top, s.top), rr = Math.min(r.right, s.right), bb = Math.min(r.bottom, s.bottom); r = { left: l, top: t, width: Math.max(0, rr - l), height: Math.max(0, bb - t), right: rr, bottom: bb }; }
      const pad = 8;
      // follow the element's own rounding (pills stay pills); radius is in zoomed px
      const br = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
      const rad = Math.min(br * zoom, Math.min(r.width, r.height) / 2) + pad;
      return [r.left - sr.left - pad, r.top - sr.top - pad, r.width + pad * 2, r.height + pad * 2, rad];
    });
    const p = pin.getBoundingClientRect();
    rects.push([p.left - sr.left - 3, p.top - sr.top - 3, p.width + 6, p.height + 6, p.width / 2 + 3]);
    const d = document.getElementById('dismiss').getBoundingClientRect();
    rects.push([d.left - sr.left - 3, d.top - sr.top - 3, d.width + 6, d.height + 6, d.width / 2 + 3]);
  }
  rects = rects.filter((r) => r[2] > 0 && r[3] > 0).map((r) => r.map((n) => Math.round(n)));
  // how much the card has to show, measured once at its natural (unfolded) size; a card whose fold state is wrong switches, no motion
  if (stage.dataset.tall === undefined && !stage.classList.contains('compact') && document.querySelector('#surface [data-hit]')) {
    stage.dataset.tall = stage.getBoundingClientRect().height / zoom > lim('compact-if-taller', 380) ? '1' : '0';
  }
  if (!open && !stage.querySelector('.revealing') && stage.classList.contains('compact') !== FOLDS()) { setView({}); return; }
  const glass = glassRects(1);
  const msg = { t: 'layout', w: Math.ceil(w0), h: Math.ceil(h0), rects, ...(morphing || !entered ? {} : { glass }), settled: !!document.querySelector('#surface [data-hit]') };   // no content yet (cold start): the host waits instead of showing an empty shell   // reaching here = the fit loop is done
  const key = JSON.stringify(msg);
  if (key !== lastSent) { lastSent = key; send(msg); }
}

export function measureSoon() {
  if (queued) return;
  queued = true;
  requestAnimationFrame(measure);
}

new ResizeObserver(measureSoon).observe(document.getElementById('surface'));

// ---------- hover (pauses expiry) ----------
stage.addEventListener('pointerenter', () => send({ t: 'hover', v: true }));
stage.addEventListener('pointerleave', () => send({ t: 'hover', v: false }));

// ---------- hold-to-drag ----------
let hold = null, suppressUntil = 0;

stage.addEventListener('pointerdown', (e) => {
  if (e.button !== 0 || e.target.closest('[data-nodrag]')) return;
  const x0 = e.clientX, y0 = e.clientY;
  const cancel = () => { clearTimeout(hold.t); removeEventListener('pointermove', move); removeEventListener('pointerup', cancel); hold = null; };
  const move = (m) => { if (Math.hypot(m.clientX - x0, m.clientY - y0) > lim('move-slop', 7)) cancel(); };
  hold = { t: setTimeout(() => {
    cancel();
    stage.classList.add('dragging');
    measureSoon();
    suppressUntil = Infinity;
    send({ t: 'drag' });      // host runs the native move loop; 'dropped' comes back when you let go
  }, e.target.closest('button, .item.act, canvas, input') ? lim('hold-ms-control', 220) : lim('hold-ms', 100)) };
  addEventListener('pointermove', move);
  addEventListener('pointerup', cancel);
});

export function dropped() {
  pin.classList.remove('holding');
  stage.classList.remove('dragging');
  measureSoon();
  suppressUntil = performance.now() + 200;   // swallow the click that follows a drag
  send({ t: 'hover', v: stage.matches(':hover') });
}

addEventListener('click', (e) => { if (performance.now() < suppressUntil) { e.stopPropagation(); e.preventDefault(); } }, true);

// ---------- dismiss + pin ----------
addEventListener('contextmenu', (e) => { e.preventDefault(); send({ t: 'dismiss' }); });

let pinned = false, pinHeld = false, pinTimer = 0;

// Pin: tap = pin/unpin. Press and HOLD = pick the whole window up and carry it (host fires drag_start / drag_end events).
pin.addEventListener('pointerdown', (e) => {
  if (e.button !== 0) return;
  pinHeld = false;
  pinTimer = setTimeout(() => {
    pinHeld = true;
    pin.classList.add('holding');
    stage.classList.add('dragging');
    measureSoon();                       // widen the window shape so the drag glow isn't clipped
    suppressUntil = Infinity;
    send({ t: 'drag', src: 'pin' });
  }, 100);   // 0.1 s: quick pick-up
});
const pinRelease = () => { clearTimeout(pinTimer); pin.classList.remove('holding'); };
pin.addEventListener('pointerup', pinRelease);
pin.addEventListener('pointerleave', pinRelease);
pin.addEventListener('click', () => {
  if (pinHeld) { pinHeld = false; return; }   // that was a carry, not a tap
  pinned = !pinned;
  stage.classList.toggle('pinned', pinned);
  pin.title = pinned ? 'Pinned. Click to unpin' : 'Keep this open';
  send({ t: 'pin', v: pinned });
});

// Dismiss just this one (counts as "not interested"). Ctrl+Alt+X dismisses everything (host hotkey).
document.getElementById('dismiss').addEventListener('click', () => send({ t: 'dismiss' }));
