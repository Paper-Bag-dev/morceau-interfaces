// Page <-> host plumbing: messages, window shape, hover, hold-to-drag, pin, dismiss.

// Minimum / maximum sizes and timings are DATA: ui/limits.json, sent by the host (overridable in prefs/ui.json). The numbers in lim(...) calls are only fallbacks.
let limits = {};
export function setLimits(l) { limits = l || {}; }
const lim = (k, d) => (typeof limits[k] === 'number' ? limits[k] : d);
export const limit = lim;   // read a limit by name from other modules (main.js uses it for the attention pulse)
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;   // no hover morph for people who asked for less motion

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
  // compact by default: a card shows at `compact` of its size until hovered (then it grows to full, below)
  applyZoom(Math.max(lim('min-zoom', 0.85), (view.scale || 1) * (grownFrom ? 1 : COMPACT())));
  // squeezed: lists show their first rows with the next one fading out, the plate gets a solid accent edge (plate.css #stage.compact)
  stage.classList.toggle('compact', COMPACT() < 0.99 && !grownFrom);
  measureSoon();
}

// ---------- hover-grow: a small 3D / chart / graph window gets a little bigger while the pointer is on it ----------
let grownFrom = 0, growTimer = 0, shrinkTimer = 0, lastShrinkAt = 0;   // lastShrinkAt: after a shrink, no re-grow for a while (the pointer can sit between the small and the grown edge and make the card flap)
const GROW = () => lim('hover-grow', 1.25), SMALL_W = () => lim('hover-small-w', 560);
const COMPACT = () => Math.min(1, lim('compact', 1));   // < 1: every card is compact until hovered, and hover grows it to full size   // grows only when the window is narrower than SMALL_W (logical px); never past the screen cap
stage.addEventListener('pointerenter', () => {
  clearTimeout(shrinkTimer);
  const any = COMPACT() < 0.99;   // compact cards grow on hover whatever they hold; otherwise only 3D / graph / chart windows do
  if (grownFrom || stage.classList.contains('dragging') || (!any && !stage.querySelector('.c-model, .c-graph, .c-chart canvas'))) return;
  clearTimeout(growTimer);
  growTimer = setTimeout(() => {
    const w = stage.getBoundingClientRect().width;   // logical px, includes zoom
    if (grownFrom || (!any && w >= SMALL_W()) || Date.now() - lastShrinkAt < lim('hover-regrow-cooldown-ms', 4000)) return;
    const room = Math.min(view.maxw / w, view.maxh / stage.getBoundingClientRect().height);
    const k = Math.min(any ? Math.max(1, (view.scale || 1) / zoom) : GROW(), room);   // compact: back to its full size
    if (k < 1.06) return;                            // no room worth growing into
    grownFrom = zoom;
    fits = 0;
    const z1 = zoom, z2 = Math.min(lim('max-zoom', 2.2), zoom * k);
    stage.classList.remove('compact');   // show everything that was tucked away
    applyZoom(z2);
    measure();   // tell the host NOW: the window is already full size, then the card eases up to it from its old size
    morph(z1 / z2, 1);
    measureSoon();
  }, lim('hover-delay-ms', 220));                    // a pass-over does not trigger it
});
stage.addEventListener('pointerleave', () => {
  clearTimeout(growTimer);
  if (!grownFrom) return;
  clearTimeout(shrinkTimer);
  shrinkTimer = setTimeout(() => {                   // linger a moment so the edge does not flicker
    const z = grownFrom; grownFrom = 0; lastShrinkAt = Date.now();
    fits = 0;
    // ease the card down inside the still-large window, then shrink the window and drop the transform
    morph(1, z / zoom, () => { stage.classList.toggle('compact', COMPACT() < 0.99); applyZoom(z); measure(); measureSoon(); });
  }, 450);
});

// Smooth grow/shrink: scale the whole surface from one factor to another (top-left origin) so hover growth is an animation, not a jump.
// A size change that the HOST asks for (e.g. a card yields room to the one in use) animates with the same default motion as hover growth, so the intent
// shows: shrinking eases down first and then the window follows; growing makes the window big first and eases up to it.
export function setViewSmooth(v) {
  const target = Math.max(lim('min-zoom', 0.85), v.scale || 1), z1 = zoom;
  if (Math.abs(target - z1) / z1 < 0.01) { setView(v); return; }
  if (target < z1) morph(1, target / z1, () => setView(v));
  else { setView(v); measure(); morph(z1 / zoom, 1); }
}

function morph(from, to, done) {
  const surf = document.getElementById('surface');
  if (!surf || reduced) { if (done) done(); return; }
  const a = surf.animate([{ transformOrigin: '0 0', transform: `scale(${from})` }, { transformOrigin: '0 0', transform: `scale(${to})` }], { duration: lim('morph-ms', 220), easing: 'cubic-bezier(.2,.8,.2,1)', fill: 'forwards' });
  a.onfinish = () => { a.cancel(); if (done) done(); };
}

function measure() {
  queued = false;
  // Never measure mid-animation: transforms (the pop-in, hover lifts) shift the rects, and the window shape would
  // lock to that moment. Wait for finite animations in the surface to end, then measure the resting layout.
  const running = document.getAnimations().filter((a) => a.playState === 'running' && a.effect && a.effect.getTiming().iterations !== Infinity && stage.contains(a.effect.target));
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
  if (fits < 5 && (k < 1 || (view.fill && k > 1.04 && zoom < lim('max-zoom', 2.2)))) {
    const next = Math.max(lim('min-zoom', 0.85), Math.min(lim('max-zoom', 2.2), zoom * k * 0.97));
    if (Math.abs(next - zoom) / zoom > 0.015) { fits++; applyZoom(next); return measureSoon(); }
  }
  let rects;
  if (stage.classList.contains('urgent') || stage.classList.contains('dragging')) {
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
  // each card plate's exact rounded rect: where the host may put native glass (macOS); the page draws the tint, grain and sheen on top
  const glass = [...document.querySelectorAll('.card-plate')].map((el) => {
    const r = el.getBoundingClientRect();
    const br = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
    return [r.left - sr.left, r.top - sr.top, r.width, r.height, Math.min(br * zoom, Math.min(r.width, r.height) / 2)].map((n) => Math.round(n));
  }).filter((r) => r[2] > 0 && r[3] > 0);
  const msg = { t: 'layout', w: Math.ceil(w0), h: Math.ceil(h0), rects, glass, settled: !!document.querySelector('#surface [data-hit]') };   // no content yet (cold start): the host waits instead of showing an empty shell   // reaching here = the fit loop is done
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
