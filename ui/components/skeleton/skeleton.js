import { registerView, html } from '../../core/view.js';

// {type:"skeleton", w:300, h:110, variant:"chart|stat|bar|list|btn|text"}
// A placeholder card the size of the component that will replace it, so the layout does not jump when the real thing arrives.
// `morc stream` makes one for every pending piece and swaps each in place; use them yourself for anything slow (a report, a graph). Preact view.
registerView('skeleton', ({ c, host }) => {
  host.style.width = (c.w || 200) + 'px';
  host.style.height = (c.h || 28) + 'px';
  return html`<span class="sk-shine"></span>`;
}, { cls: (c) => 'sk ' + (c.variant || 'text') });
