// Motion defaults: everything arrives with a dampened spring and leaves quickly.
// Springs are simulated once into Web Animations keyframes, so the browser animates them off the main thread.

/** Simulate a damped spring 0 -> 1. Returns {values: number[], ms}. */
export function simulate({ stiffness = 320, damping = 24, mass = 1 } = {}) {
  const dt = 1 / 60;
  let x = 0, v = 0, t = 0;
  const values = [0];
  while (t < 1.2) {
    const a = (-stiffness * (x - 1) - damping * v) / mass;
    v += a * dt;
    x += v * dt;
    t += dt;
    values.push(x);
    if (Math.abs(x - 1) < 0.0015 && Math.abs(v) < 0.01) break;
  }
  values[values.length - 1] = 1;
  return { values, ms: Math.round(t * 1000) };
}

const SPRING = simulate();

/** Arrive: a card appears from nothing, scales up on a damped spring (a little overshoot, then it settles) and rests. The spring and the
 *  start size come from limits (arrive-from, arrive-stiffness, arrive-damping); returns the animation (its duration drives the glass). */
// The growth follows the card's shape so it reads clearly (user, 11-11: big cards just appeared): a WIDE card (wider than `arrive-wide`
// x its height) unfolds left to right, a LONG card (taller than `arrive-long` x its width) unrolls top to bottom, anything else grows
// from `arrive-from` of its size around its centre. The unfolds also scale a little on the cross axis so they feel like growth, not a wipe.
export function arrive(el, lim) {
  const from = lim('arrive-from', 0.08);
  const sp = simulate({ stiffness: lim('arrive-stiffness', 230), damping: lim('arrive-damping', 19) });
  const r = el.getBoundingClientRect();
  const wide = r.width > r.height * lim('arrive-wide', 1.6), long = r.height > r.width * lim('arrive-long', 1.15);
  const n = sp.values.length;
  const kf = sp.values.map((p, i) => {
    const o = Math.min(1, (i / n) * 6), q = Math.max(0, p);   // q: no negative overshoot into an inside-out clip
    if (wide) return { opacity: o, clipPath: `inset(0 ${Math.max(0, (1 - q) * 100)}% 0 0 round 16px)`, transform: `scaleY(${0.86 + 0.14 * p})`, transformOrigin: '0 50%' };
    if (long) return { opacity: o, clipPath: `inset(0 0 ${Math.max(0, (1 - q) * 100)}% 0 round 16px)`, transform: `scaleX(${0.9 + 0.1 * p})`, transformOrigin: '50% 0' };
    return { opacity: o, transform: `scale(${from + (1 - from) * p})`, transformOrigin: '50% 50%' };
  });
  return el.animate(kf, { duration: sp.ms, easing: 'linear', fill: 'backwards' });
}

/** Rows of a list, checklist, paste set or ask set come in one after another once the card is open (`row-stagger-ms` apart, max 12). */
export function rows(root, lim) {
  const step = lim('row-stagger-ms', 45), start = lim('row-start-ms', 160);
  const items = root.querySelectorAll(':is(.c-list, .c-check, .c-pastes, .c-asks) > *');
  [...items].slice(0, 12).forEach((it, i) => it.animate(
    [{ opacity: 0, transform: 'translateY(8px)' }, { opacity: 1, transform: 'none' }],
    { duration: 260, delay: start + i * step, easing: 'cubic-bezier(.2,.8,.2,1)', fill: 'backwards' }));
}

/** Pop in: slide up a little, scale from .92, fade. `delay` staggers siblings. */
export function enter(el, delay = 0) {
  const kf = SPRING.values.map((p) => ({
    opacity: Math.min(1, p * 2.2),
    transform: `translateY(${(1 - p) * 14}px) scale(${0.92 + 0.08 * p})`,
  }));
  return el.animate(kf, { duration: SPRING.ms, delay, easing: 'linear', fill: 'backwards' });
}

/** Quick exit: no bounce, so it never feels like it is arguing with you. */
export function leave(el) {
  return el.animate(
    [{ opacity: 1, transform: 'translateY(0) scale(1)' }, { opacity: 0, transform: 'translateY(6px) scale(.96)' }],
    { duration: 170, easing: 'cubic-bezier(.4,0,1,1)', fill: 'forwards' },
  );
}

/** Attention pulse for summoned/urgent surfaces. */
export function pulse(el) {
  return el.animate([{ transform: 'scale(1)' }, { transform: 'scale(1.045)' }, { transform: 'scale(1)' }], { duration: 520, easing: 'cubic-bezier(.34,1.56,.64,1)' });
}

/** Ease a number toward a target each frame (critically damped feel). Returns stop(). */
export function glide(get, set, target, rate = 14) {
  let raf, last = performance.now();
  const step = (now) => {
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    const cur = get();
    const next = cur + (target() - cur) * (1 - Math.exp(-rate * dt));
    set(Math.abs(next - target()) < 0.002 ? target() : next);
    if (next !== target()) raf = requestAnimationFrame(step);
  };
  raf = requestAnimationFrame(step);
  return () => cancelAnimationFrame(raf);
}

// Live binding: importers see changes. Reduced when the OS asks, or when the annoyance vector's `motion` is low.
const OS_REDUCED = matchMedia('(prefers-reduced-motion: reduce)').matches;
export let reduced = OS_REDUCED;
export function setReduced(v) { reduced = OS_REDUCED || !!v; }
