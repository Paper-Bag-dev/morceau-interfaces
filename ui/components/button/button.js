import { register } from '../../core/registry.js';
import { reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';
import { icon } from '../../core/icons.js';
import * as literacy from '../../core/literacy.js';

// {type:"button", label?, icon?, variant:"primary|secondary|ghost|good|bad|destructive", shape?:"soft|circle", title?, action}
//
// Clarity metric (icons): icon + label until the user has used that icon twice ("got" it), then icon-only with the
// label as a tooltip. Yes/no use check/x with good/bad so the answer is read by shape and color, not by words.
// Feedback: every click gives an immediate response (ripple + press-bounce, and a tick that morphs in for non-closing
// actions) on top of the causal change the action itself makes. The linter (src/lint.rs) refuses dead buttons.

const LEARN_AFTER = 8;   // uses before an icon is trusted to stand alone: slow on purpose, a "+" or a check is easy to misread early

register('button', (c, ctx) => {
  const b = document.createElement('button');
  b.dataset.hit = '';
  const kind = c.icon ? `icon:${c.icon}` : '';
  let showLabel = true;
  if (c.icon && c.label && literacy.get(kind).got >= LEARN_AFTER) showLabel = false;      // learned: icon alone is enough
  if (c.icon && !c.label) showLabel = false;
  const iconOnly = !!c.icon && !showLabel;
  b.className = ['btn', c.variant || 'secondary', c.shape || '', iconOnly ? 'icon-only' : ''].join(' ').replace(/\s+/g, ' ').trim();

  if (c.swatch) { b.classList.add('swatch'); b.style.setProperty('--sw', c.swatch); }                  // a colour dot (needs a title): repaint/choose-a-colour controls
  if (c.pressed) ctx.bind(c.pressed, (v) => b.classList.toggle('on', c.pressed.eq !== undefined ? v === c.pressed.eq : !!v && v !== '0'));   // {bind, eq} = selected when the value equals eq                  // shows the state of a toggled value
  const ic = c.icon ? icon(c.icon, iconOnly ? 15 : 13) : null;
  const lb = document.createElement('span');
  lb.className = 'lb';
  let labelText = '';
  ctx.bind(c.label, (v) => { labelText = v ?? ''; lb.textContent = labelText; b.title = c.title || (iconOnly ? labelText : ''); });
  if (!c.label) b.title = c.title || c.icon || '';
  b.setAttribute('aria-label', c.title || labelText || c.icon || 'button');
  if (ic) b.append(ic);
  if (showLabel && c.label) b.append(lb);

  // Non-user-triggered motion is spent sparingly: the shimmer sweep only plays on an IMPORTANT ask's affirmative button (opts.importance high/critical).
  if ((c.variant === 'primary' || c.variant === 'good') && !reduced && document.getElementById('stage').classList.contains('pulse')) {
    const s = document.createElement('span');
    s.className = 'shine';
    b.append(s);
    setTimeout(() => s.animate([{ transform: 'translateX(-120%) skewX(-18deg)' }, { transform: 'translateX(260%) skewX(-18deg)' }], { duration: 900, easing: 'ease-in-out' }), 350);
  }

  // the "done" tick that morphs in after a click that doesn't close the surface
  function confirmTick() {
    if (reduced) return;
    const tick = icon('check', 14);
    tick.classList.add('tick');
    b.append(tick);
    tick.animate([{ transform: 'scale(0) rotate(-40deg)', opacity: 0 }, { transform: 'scale(1.25) rotate(0)', opacity: 1, offset: .55 }, { transform: 'scale(1)', opacity: 1 }], { duration: 380, easing: 'cubic-bezier(.34,1.56,.64,1)', fill: 'forwards' });
    setTimeout(() => tick.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 200, fill: 'forwards' }).finished.then(() => tick.remove(), () => tick.remove()), 900);
  }

  b.addEventListener('click', (e) => {
    rippleAt(b, e);
    if (!reduced) b.animate([{ transform: 'scale(.93)' }, { transform: 'scale(1.04)' }, { transform: 'scale(1)' }], { duration: 360, easing: 'cubic-bezier(.34,1.56,.64,1)' });
    if (c.icon) literacy.mark(kind, 'got');                    // used it: counts toward dropping the label
    const acts = Array.isArray(c.action) ? c.action : [c.action];
    if (!acts.some((a) => a && a.do === 'close')) confirmTick();
    ctx.act(c.action);
  });
  return b;
});
