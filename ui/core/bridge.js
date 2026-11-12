import { simulate } from './motion.js';
import * as compact from './compact.js';
// Page <-> host plumbing: messages, window shape, hover, hold-to-drag, pin, dismiss.

// Minimum / maximum sizes and timings are DATA: ui/limits.json, sent by the host (overridable in prefs/ui.json). The numbers in lim(...) calls are only fallbacks.
let limits = {};
export function setLimits(l) { limits = l || {}; }
const lim = (k, d) => (typeof limits[k] === 'number' ? limits[k] : d);
export const limit = lim;   // read a limit by name from other modules (main.js uses it for the attention pulse)
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;   // no hover morph for people who asked for less motion

// A linked card (opened by the root page, src/surface/root.rs) shares the root's ipc channel: its key tells the host which card spoke.
const cardKey = new URLSearchParams(location.search).get('k');
export const send = (m) => window.ipc && window.ipc.postMessage(cardKey ? 'k' + cardKey + '\n' + JSON.stringify(m) : JSON.stringify(m));

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
  // the card's own buttons keep ONE size whatever the card's zoom (nested zoom multiplies: 1/z cancels it)
  const ctl = document.getElementById('controls');
  if (ctl) ctl.style.zoom = String(1 / z);
  const surf = document.getElementById('surface');
  if (surf) surf.style.maxWidth = view.wrap > 0 ? Math.round(view.wrap / z - 52) + 'px' : '';   // long text wraps to the gap beside the neighbour
  window.dispatchEvent(new Event('morc-zoom'));   // charts repaint at the right resolution
}

// The card's own buttons on an arc around the plate's top-right corner: the centre of that corner's curve is the circle's centre; each button
// sits --ctl-gap outside the edge, at 90+spread deg (pin, on the top edge side), 45 deg (x, the diagonal), -spread deg (layer, right edge
// side). Measured in screen px (the controls cancel the card's zoom), so they hug the plate at any size or zoom. Falls back to the
// content's box when the card has no plate (a 3D model, bare text).
function placeControls() {
  const ctl = document.getElementById('controls');
  const plate = document.querySelector('#surface .card-plate') || document.querySelector('#surface > *');
  if (!ctl || !plate) return;
  const cs = getComputedStyle(stage), num = (n, d) => parseFloat(cs.getPropertyValue(n)) || d;
  const size = num('--ctl-size', 24), gap = num('--ctl-gap', 6), zone = num('--ctl-zone', 26);
  const spread = (num('--ctl-spread', 22) * Math.PI) / 180;
  // plate corner in screen px from the stage's top-right (offset* are layout px, unaffected by transforms; x zoom = screen px)
  const z = zoom;
  let x = 0, y = 0;
  for (let e = plate; e && e !== stage; e = e.offsetParent) { x += e.offsetLeft; y += e.offsetTop; }
  const right = (stage.clientWidth - (x + plate.offsetWidth)) * z, top = y * z;
  const R = Math.min(parseFloat(getComputedStyle(plate).borderTopRightRadius) || 0, plate.offsetWidth / 2, plate.offsetHeight / 2) * z;
  const r = R + gap + size / 2;                        // circle the button centres run on
  const cx = right + R, cy = top + R;                  // corner curve centre, measured from the stage's top-right
  const at = (deg) => [cx - r * Math.cos(deg) - size / 2, cy - r * Math.sin(deg) - size / 2];   // [right, top] of a button
  const spots = { pin: at(Math.PI / 2 + spread), dismiss: at(Math.PI / 4), layer: at(-spread) };
  let w = 0, h = 0;
  for (const [id, [rr, tt]] of Object.entries(spots)) {
    const b = document.getElementById(id);
    b.style.right = Math.round(rr) + 'px';
    b.style.top = Math.round(tt) + 'px';
    w = Math.max(w, rr + size); h = Math.max(h, tt + size);
  }
  ctl.style.setProperty('--ctl-zone-w', Math.round(w + zone) + 'px');   // the hot zone: the corner out to the buttons, plus a reach
  ctl.style.setProperty('--ctl-zone-h', Math.round(h + zone) + 'px');
}

export function setView(v) {
  view = { ...view, ...v };
  fits = 0;
  // compact by default: a card shows at `compact` of its size until hovered (then it grows to full, below)
  // medium (info-dense working cards: pastes, to-dos, asks) start a little bigger (limits medium-scale); large (charts, graphs) bigger still
  const sizeK = stage.dataset.size === 'medium' ? lim('medium-scale', 1.12) : stage.dataset.size === 'large' ? lim('large-scale', 1.3) : 1;   // charts and graphs start big
  applyZoom(Math.max(lim('min-zoom', 0.85), (view.scale || 1) * sizeK * (grownFrom ? 1 : COMPACT())));
  // squeezed: lists show their first rows with the next one fading out, the plate gets a solid accent edge (plate.css #stage.compact)
  setCompact(FOLDS() && !grownFrom);
  measureSoon();
}

// ---------- hover-grow: a small 3D / chart / graph window gets a little bigger while the pointer is on it ----------
let grownFrom = 0, growTimer = 0, shrinkTimer = 0, lastShrinkAt = 0;   // lastShrinkAt: after a shrink, no re-grow for a while (the pointer can sit between the small and the grown edge and make the card flap)
const GROW = () => lim('hover-grow', 1.25), SMALL_W = () => lim('hover-small-w', 560);
// < 1: a LIST-LIKE card (rows to tuck away) starts compact and grows to full size on hover. Charts, graphs, stats and the rest keep their
// size (their small state is already big) and just grow a little on hover, like before (user, 11-10).
const LISTY = '.c-list, .c-pastes, .c-check', VISUAL = '.c-model, .c-graph, .c-chart canvas';
// The chip can say what it is (opts.size, set on #stage as data-size by core/main.js): "compact" = a glance-and-go list (starts small),
// "medium" = a working card you read or use (pastes, to-dos, asks: never squeezed), "large" = charts and graphs (big from the start).
// No size: a plain list (.c-list) starts compact, everything else at full size.
const COMPACT = () => {
  const sz = stage.dataset.size;
  // a card with a lot to show (its natural height past limits.compact-if-taller) starts compact and springs open on hover, whatever its
  // kind, charts aside (user, 11-10: long cards like the asks follow the compact + damped-spring way)
  if (stage.dataset.tall === '1' && sz !== 'large' && !stage.querySelector(VISUAL)) return Math.min(1, lim('compact', 1));
  if (sz === 'medium' || sz === 'large') return 1;
  if (sz === 'compact' || (stage.querySelector('.c-list') && !stage.querySelector(VISUAL))) return Math.min(1, lim('compact', 1));
  return 1;
};   // grows only when the window is narrower than SMALL_W (logical px); never past the screen cap
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
    compact.expand(stage, REVEAL());   // show everything that was tucked away (slides open)
    stage.classList.add('grown');      // charts show their values while grown
    morphing = true;     // the measure below sizes the window; the glass waits for the spring to drive it
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
    // the mirror of growing: the content folds (lists slide shut, width eases in, the edge thickens) WHILE the card springs down inside
    // the still-large window; only when both have settled does the zoom take over from the transform and the window shrink to fit, and
    // since the card already looks exactly like that, nothing jumps
    const fold = FOLDS();
    if (fold) compact.collapse(stage, REVEAL());
    stage.classList.remove('grown');
    const settle = () => { applyZoom(z); measure(); measureSoon(); };
    let left = 2;
    const both = () => { if (--left === 0) settle(); };
    morph(1, z / zoom, both);
    setTimeout(both, fold && !reduced ? REVEAL() + 40 : 0);
  }, lim('shrink-delay-ms', 450));
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

// a damped spring (limits spring-stiffness / spring-damping): settles with the barest overshoot, so growth feels physical, not linear
let SPRING_K = null, morphing = false, entered = false;   // entered: no native glass before the card's arrival starts (it flashed full size)
function spring() {
  const k = `${lim('spring-stiffness', 260)}|${lim('spring-damping', 28)}`;
  if (!SPRING_K || SPRING_K.key !== k) SPRING_K = { key: k, ...simulate({ stiffness: lim('spring-stiffness', 260), damping: lim('spring-damping', 28) }) };
  return SPRING_K;
}
/** The native glass follows the card through an animation it did not start (the arrival): the plate's live rect every frame until `anim` ends. */
export function followGlass(anim) {
  entered = true;
  if (!anim) { measure(); return; }
  morphing = true;
  const tick = () => { if (anim.playState === 'finished' || anim.playState === 'idle') { morphing = false; measure(); return; } send({ t: 'glass', glass: glassRects(1) }); requestAnimationFrame(tick); };
  requestAnimationFrame(tick);
}

function morph(from, to, done) {
  const surf = document.getElementById('surface');
  if (!surf || reduced) { if (done) done(); return; }
  const sp = spring();
  const kf = sp.values.map((p) => ({ transformOrigin: '0 0', transform: `scale(${from + (to - from) * p})` }));
  const a = surf.animate(kf, { duration: sp.ms, easing: 'linear', fill: 'forwards' });
  morphing = true;
  // the native glass (macOS) follows the spring: the plate's live rect every frame (getBoundingClientRect includes the transform)
  const follow = () => { if (!morphing) return; send({ t: 'glass', glass: glassRects(1) }); requestAnimationFrame(follow); };
  requestAnimationFrame(follow);
  a.onfinish = () => { morphing = false; a.cancel(); if (done) done(); else measure(); };
}

// each card plate's exact rounded rect: where the host may put native glass (macOS); the page draws the tint, grain and sheen on top.
// f scales them from the top-left (the morph's origin), for moving the glass ahead of a shrink.
function glassRects(f) {
  const sr = stage.getBoundingClientRect();
  return [...document.querySelectorAll('.card-plate')].map((el) => {
    const r = el.getBoundingClientRect();
    const br = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
    return [(r.left - sr.left) * f, (r.top - sr.top) * f, r.width * f, r.height * f, Math.min(br * zoom, Math.min(r.width, r.height) / 2) * f].map((n) => Math.round(n));
  }).filter((r) => r[2] > 0 && r[3] > 0);
}

// compact <-> full: the content side lives in core/compact.js; this file owns the zoom and the window
const REVEAL = () => lim('reveal-ms', 380);
function setCompact(on) { compact.set(stage, on); }
// should this card be folded right now? (squeezed lists and long cards, or anything with a key part and detail beside it)
const FOLDS = () => COMPACT() < 0.99 || compact.folds(stage);
stage.addEventListener('pointerenter', () => stage.classList.add('hovered'));
stage.addEventListener('pointerleave', () => stage.classList.remove('hovered'));

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
  placeControls();
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
    for (const id of ['dismiss', 'layer']) {
      const d = document.getElementById(id).getBoundingClientRect();
      rects.push([d.left - sr.left - 3, d.top - sr.top - 3, d.width + 6, d.height + 6, d.width / 2 + 3]);
    }
    // the corner hot zone that brings the buttons out: part of the window's shape, so the pointer reaching it is seen (macOS hit test)
    const z = document.getElementById('controls').getBoundingClientRect();
    rects.push([z.left - sr.left, z.top - sr.top, z.width, z.height, 0]);
  }
  rects = rects.filter((r) => r[2] > 0 && r[3] > 0).map((r) => r.map((n) => Math.round(n)));
  // the first view can arrive before the content: once it is here, a card that should not be compact (or should) switches, no motion
  // how much the card has to show, measured once at its natural (uncompacted) size
  if (stage.dataset.tall === undefined && !stage.classList.contains('compact') && document.querySelector('#surface [data-hit]')) {
    const h = stage.getBoundingClientRect().height / zoom;
    stage.dataset.tall = h > lim('compact-if-taller', 380) ? '1' : '0';
  }
  if (!grownFrom && !morphing && stage.classList.contains('compact') !== FOLDS()) { setView({}); return; }
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

// Layer: behind the active window and back. The host owns the state (morc card <id> lower|raise, the scene tree); it answers with
// {"t":"layer","below"} which sets the look, so a CLI / MCP change shows here too.
const layerBtn = document.getElementById('layer');
// squeeze on press, for all three
for (const b of document.querySelectorAll('#controls .ctl')) {
  b.addEventListener('pointerdown', () => b.classList.add('press'));
  for (const ev of ['pointerup', 'pointerleave', 'pointercancel']) b.addEventListener(ev, () => b.classList.remove('press'));
}
layerBtn.addEventListener('click', () => send({ t: 'layer', below: !stage.classList.contains('below') }));
export function setLayer(below) {
  stage.classList.toggle('below', !!below);
  layerBtn.title = below ? 'Bring back on top' : 'Send behind the active window';
  layerBtn.setAttribute('aria-label', layerBtn.title);
}
