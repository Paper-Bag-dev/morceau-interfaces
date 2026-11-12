// Preact components and Tailwind classes on cards (docs/views.md). The engine itself has no look: colors, fonts and radii come from the
// active preset (ui/preset.css, yours to change), which maps the theme's CSS variables onto Tailwind's names.
//
//   registerView('tasklist', ({ c, useBind, act }) => html`<div class="flex flex-col gap-2">...</div>`)
//     c = the component's spec, useBind(v) = the live value of a literal / {bind} / "{/x} template" (re-renders on change),
//     act(action) = the same actions a button runs (set, push, call ...). Mounted into an element like any other component.
//   any component's spec may carry "class": "p-2 rounded-xl ..." (Tailwind utilities, written on the fly).
//
// Tailwind's browser build runs only on a card that uses a class (first use loads it); its reset (preflight) is left out, so the
// existing components keep their look. `hover:` is redefined as :is(:hover, .hov): on macOS the page cannot see the pointer while
// our app is inactive, the host marks the hovered chain with .hov instead (core/main.js markHover).

import { h, render } from 'preact';
import { useState, useEffect, useRef } from 'preact/hooks';
import htm from 'htm';
import { register } from './registry.js';
import { measureSoon } from './bridge.js';

export const html = htm.bind(h);

let tw = null;
/** Resolves once Tailwind's classes apply (at once when this card uses none). */
export const twReady = () => tw || Promise.resolve();
export function ensureTailwind() {
  if (tw) return tw;
  tw = (async () => {
    const preset = await fetch('preset.css').then((r) => (r.ok ? r.text() : '')).catch(() => '');
    const style = document.createElement('style');
    style.type = 'text/tailwindcss';
    style.textContent = `@import "tailwindcss/theme.css" layer(theme);
@import "tailwindcss/utilities.css" layer(utilities);
@custom-variant hover (&:is(:hover, .hov));
${preset}`;
    document.head.append(style);
    await new Promise((done) => {
      const s = document.createElement('script');
      s.src = 'vendor/tailwind/tailwind.browser.js';
      s.onload = s.onerror = done;
      document.head.append(s);
    });
    measureSoon();   // the new classes change sizes: re-measure once they apply
  })();
  return tw;
}

const UNSET = {};

export function registerView(type, View) {
  register(type, (c, ctx) => {
    const el = document.createElement('div');
    el.className = 'c-view';
    el.dataset.hit = '';
    function Host() {
      // the bindings are made once per mount; their callbacks feed this component's state, so a patch re-renders only what changed
      const binds = useRef(null);
      const [, tick] = useState(0);
      if (!binds.current) binds.current = new Map();
      const useBind = (v) => {
        const key = typeof v === 'object' ? JSON.stringify(v) : String(v);
        let b = binds.current.get(key);
        if (!b) {
          b = { v: UNSET };
          binds.current.set(key, b);
          ctx.bind(v, (x) => { const first = b.v === UNSET; b.v = x; if (!first) tick((n) => n + 1); });
        }
        return b.v === UNSET ? undefined : b.v;
      };
      useEffect(() => { measureSoon(); });
      return h(View, { c, ctx, useBind, act: (a) => ctx.act(a) });
    }
    render(h(Host, {}), el);
    if (JSON.stringify(c).includes('"class"') || View.tailwind !== false) ensureTailwind();
    return el;
  });
}

// A generic element for layouts written on the fly: {type:"el", tag:"div", class:"flex gap-2", text?, children?}.
// Tags are limited to plain layout and text ones: a spec can never inject script, iframe or form-posting elements.
const TAGS = new Set(['div', 'span', 'p', 'section', 'header', 'footer', 'ul', 'ol', 'li', 'h1', 'h2', 'h3', 'h4', 'strong', 'em', 'small', 'code', 'pre', 'hr']);
register('el', (c, ctx) => {
  const el = document.createElement(TAGS.has(c.tag) ? c.tag : 'div');
  if (c.text != null) ctx.bind(c.text, (v) => { el.textContent = v ?? ''; });
  else for (const k of ctx.children) el.append(k());
  if (c.action) { el.dataset.hit = ''; el.addEventListener('click', () => ctx.act(c.action)); }
  return el;
});
