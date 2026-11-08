// Page <-> host plumbing: messages, window shape, hover, hold-to-drag, pin, dismiss.

const HOLD_MS = 100;      // hold the left button this long (without moving) and the window picks up: 0.1 s on empty areas and text
const HOLD_MS_CONTROL = 220; // buttons, rows, canvases: a normal click lasts ~100 ms, so these arm a little later and still click
const MOVE_SLOP = 7;
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;   // no hover morph for people who asked for less motion
const MIN_ZOOM = 0.85;      // never shrink content below this: text, icons and charts stay readable; the window wraps/scrolls instead

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
  applyZoom(Math.max(MIN_ZOOM, view.scale || 1));
  measureSoon();
}

// ---------- hover-grow: a small 3D / chart / graph window gets a little bigger while the pointer is on it ----------
let grownFrom = 0, growTimer = 0, shrinkTimer = 0, lastShrinkAt = 0;   // lastShrinkAt: after a shrink, no re-grow for a while (the pointer can sit between the small and the grown edge and make the card flap)
const GROW = 1.25, SMALL_W = 560;   // grows only when the window is narrower than SMALL_W (logical px); never past the screen cap
stage.addEventListener('pointerenter', () => {
  clearTimeout(shrinkTimer);
  if (grownFrom || stage.classList.contains('dragging') || !stage.querySelector('.c-model, .c-graph, .c-chart canvas')) return;
  clearTimeout(growTimer);
  growTimer = setTimeout(() => {
    const w = stage.getBoundingClientRect().width;   // logical px, includes zoom
    if (grownFrom || w >= SMALL_W || Date.now() - lastShrinkAt < 4000) return;
    const room = Math.min(view.maxw / w, view.maxh / stage.getBoundingClientRect().height);
    const k = Math.min(GROW, room);
    if (k < 1.06) return;                            // no room worth growing into
    grownFrom = zoom;
    fits = 0;
    const z1 = zoom, z2 = Math.min(2.2, zoom * k);
    applyZoom(z2);
    measure();   // tell the host NOW: the window is already full size, then the card eases up to it from its old size
    morph(z1 / z2, 1);
    measureSoon();
  }, 220);                                           // a pass-over does not trigger it
});
stage.addEventListener('pointerleave', () => {
  clearTimeout(growTimer);
  if (!grownFrom) return;
  clearTimeout(shrinkTimer);
  shrinkTimer = setTimeout(() => {                   // linger a moment so the edge does not flicker
    const z = grownFrom; grownFrom = 0; lastShrinkAt = Date.now();
    fits = 0;
    // ease the card down inside the still-large window, then shrink the window and drop the transform
    morph(1, z / zoom, () => { applyZoom(z); measure(); measureSoon(); });
  }, 450);
});

// Smooth grow/shrink: scale the whole surface from one factor to another (top-left origin) so hover growth is an animation, not a jump.
function morph(from, to, done) {
  const surf = document.getElementById('surface');
  if (!surf || reduced) { if (done) done(); return; }
  const a = surf.animate([{ transformOrigin: '0 0', transform: `scale(${from})` }, { transformOrigin: '0 0', transform: `scale(${to})` }], { duration: 220, easing: 'cubic-bezier(.2,.8,.2,1)', fill: 'forwards' });
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
  if (fits < 5 && (k < 1 || (view.fill && k > 1.04 && zoom < 2.2))) {
    const next = Math.max(MIN_ZOOM, Math.min(2.2, zoom * k * 0.97));
    if (Math.abs(next - zoom) / zoom > 0.015) { fits++; applyZoom(next); return measureSoon(); }
  }
  let rects;
  if (stage.classList.contains('urgent') || stage.classList.contains('dragging')) {
    rects = [[3, 3, w0 - 6, h0 - 6, 28]];
  } else {
    rects = [...document.querySelectorAll('[data-hit]')].map((el) => {
      let r = el.getBoundingClientRect();
      // a child scrolled out of its scroll box must not leave an invisible clickable zone: clip to the scroller
      const sc = el.parentElement && el.parentElement.closest('.scroll');
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
  const msg = { t: 'layout', w: Math.ceil(w0), h: Math.ceil(h0), rects, settled: !!document.querySelector('#surface [data-hit]') };   // no content yet (cold start): the host waits instead of showing an empty shell   // reaching here = the fit loop is done
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
  const move = (m) => { if (Math.hypot(m.clientX - x0, m.clientY - y0) > MOVE_SLOP) cancel(); };
  hold = { t: setTimeout(() => {
    cancel();
    stage.classList.add('dragging');
    measureSoon();
    suppressUntil = Infinity;
    send({ t: 'drag' });      // host runs the native move loop; 'dropped' comes back when you let go
  }, e.target.closest('button, .item.act, canvas, input') ? HOLD_MS_CONTROL : HOLD_MS) };
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
