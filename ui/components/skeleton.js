import { register } from '../core/registry.js';

// {type:"skeleton", w:300, h:110, variant:"chart|stat|bar|list|btn|text"}
// A placeholder card the size of the component that will replace it, so the layout does not jump when the real thing arrives.
// `morc stream` makes one for every pending piece and swaps each in place; use them yourself for anything slow (a report, a graph).
register('skeleton', (c) => {
  const el = document.createElement('div');
  el.className = 'sk ' + (c.variant || 'text');
  el.dataset.hit = '';
  el.style.width = (c.w || 200) + 'px';
  el.style.height = (c.h || 28) + 'px';
  const shine = document.createElement('span');
  shine.className = 'sk-shine';
  el.append(shine);
  return el;
});