import { registerView, html } from '../../core/view.js';

// text, badge, kv: Preact views (docs/views.md). The host element keeps the classes the skins style (.c-text, .badge, .c-kv); a bound
// tone swaps the tone-* class on the host.
const toneCls = (t) => (t && typeof t === 'string' ? 'tone-' + t : '');

registerView('text', ({ c, useBind, host }) => {
  const text = useBind(c.text);
  const tone = c.tone && typeof c.tone === 'object' ? useBind(c.tone) : c.tone;
  host.className = ['c-text', c.size || '', c.bold ? 'b' : '', toneCls(tone)].filter(Boolean).join(' ');
  return html`${text ?? ''}`;
}, { cls: 'c-text' });

registerView('badge', ({ c, useBind }) => html`${useBind(c.text) ?? ''}`, { tag: 'span', cls: (c) => 'badge inline-flex items-center' + (c.tone ? ' tone-' + c.tone : '') });

registerView('kv', ({ c, useBind }) => html`<span class="k">${useBind(c.k) ?? ''}</span><span class="min-w-0">${useBind(c.v) ?? ''}</span>`,
  { cls: 'c-kv' });
