import { registerView, html, Icon, useEffect, useRef } from '../../core/view.js';
import { reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';
import { icon } from '../../core/icons.js';
import * as literacy from '../../core/literacy.js';

// {type:"button", label?, icon?, variant:"primary|secondary|ghost|good|bad|destructive", shape?:"soft|circle", title?, action, tone?, swatch?, pressed?}
// Preact view; the host IS the <button>, so the skins' .btn rules apply unchanged.
//
// Clarity metric (icons): icon + label until the user has used that icon a few times ("got" it), then icon-only with the
// label as a tooltip. Yes/no use check/x so the answer is read by shape, not words.
// Feedback: every click gives an immediate response (ripple + press-bounce, and a tick that morphs in for non-closing
// actions) on top of the causal change the action itself makes. The linter (src/lint.rs) refuses dead buttons.

const LEARN_AFTER = 8;   // uses before an icon is trusted to stand alone: slow on purpose, a "+" or a check is easy to misread early

const btnCls = (c) => {
  const kind = c.icon ? `icon:${c.icon}` : '';
  const iconOnly = !!c.icon && (!c.label || literacy.get(kind).got >= LEARN_AFTER);
  return ['btn', c.variant || 'secondary', c.shape || '', iconOnly ? 'icon-only' : '', c.swatch ? 'swatch' : ''].filter(Boolean).join(' ');
};

// the "done" tick that morphs in after a click that doesn't close the surface (plain DOM: it lives 1 s and removes itself)
function confirmTick(b) {
  if (reduced) return;
  const tick = icon('check', 14);
  tick.classList.add('tick');
  b.append(tick);
  tick.animate([{ transform: 'scale(0) rotate(-40deg)', opacity: 0 }, { transform: 'scale(1.25) rotate(0)', opacity: 1, offset: .55 }, { transform: 'scale(1)', opacity: 1 }], { duration: 380, easing: 'cubic-bezier(.34,1.56,.64,1)', fill: 'forwards' });
  setTimeout(() => tick.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 200, fill: 'forwards' }).finished.then(() => tick.remove(), () => tick.remove()), 900);
}

registerView('button', ({ c, ctx, useBind, host }) => {
  const kind = c.icon ? `icon:${c.icon}` : '';
  const label = useBind(c.label) ?? '';
  const iconOnly = host.classList.contains('icon-only');
  const pv = c.pressed ? useBind(c.pressed) : undefined;
  if (c.pressed) host.classList.toggle('on', c.pressed.eq !== undefined ? pv === c.pressed.eq : !!pv && pv !== '0');   // {bind, eq} = selected when the value equals eq
  // tone (user, 11-11): a colour by name from themes.json base tone-*; no tone or "accent" = the accent. The colour is data, never typed here.
  if (c.tone && c.tone !== 'accent') { host.dataset.tone = c.tone; host.style.setProperty('--btn-tone', `var(--tone-${c.tone})`); }
  if (c.swatch) host.style.setProperty('--sw', c.swatch);
  host.title = c.title || label || c.icon || '';   // the label is cut to one line, so the full text is always in the tooltip
  host.setAttribute('aria-label', c.title || label || c.icon || 'button');

  const wired = useRef(false);
  useEffect(() => {
    if (wired.current) return;
    wired.current = true;
    host.addEventListener('click', (e) => {
      rippleAt(host, e);
      if (!reduced) host.animate([{ transform: 'scale(.93)' }, { transform: 'scale(1.04)' }, { transform: 'scale(1)' }], { duration: 360, easing: 'cubic-bezier(.34,1.56,.64,1)' });
      if (c.icon) literacy.mark(kind, 'got');                   // used it: counts toward dropping the label
      const acts = Array.isArray(c.action) ? c.action : [c.action];
      if (!acts.some((a) => a && a.do === 'close')) confirmTick(host);
      ctx.act(c.action);
    });
    // non-user-triggered motion is spent sparingly: the shimmer sweep only plays on an IMPORTANT ask's affirmative button
    if ((c.variant === 'primary' || c.variant === 'good') && !reduced && document.getElementById('stage').classList.contains('pulse')) {
      const s = host.querySelector('.shine');
      if (s) setTimeout(() => s.animate([{ transform: 'translateX(-120%) skewX(-18deg)' }, { transform: 'translateX(260%) skewX(-18deg)' }], { duration: 900, easing: 'ease-in-out' }), 350);
    }
  }, []);

  return html`
    ${c.icon ? html`<${Icon} name=${c.icon} size=${iconOnly ? 15 : 13} />` : null}
    ${c.label && !iconOnly ? html`<span class="lb min-w-0 truncate">${label}</span>` : null}
    ${c.variant === 'primary' || c.variant === 'good' ? html`<span class="shine"></span>` : null}`;
}, { tag: 'button', cls: btnCls });
