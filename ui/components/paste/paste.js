import { register } from '../../core/registry.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';
import { icon } from '../../core/icons.js';

// {type:"pastes", items:{bind:"/items"}, lines?:12, max_frac?:0.6}   item: {title, text, note?}
// The list never grows past max_frac of the screen height: past that it scrolls inside the card, so every row and its Copy button stays reachable.
// Paste blocks to hand over in order: one numbered row each with its size and a Copy button. A click on the row shows the exact text
// (scrollable, selectable); Copy puts it on the clipboard (the host does it with pbcopy on macOS, see surface/actions.rs) and the row
// stays marked "Copied", so you can see which ones are done.
register('pastes', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'c-pastes';
  el.dataset.hit = '';
  // screen.availHeight is in CSS px, the same unit as the card; the window itself is sized to its content, so vh would be circular
  el.style.maxHeight = Math.round((screen.availHeight || 900) * (c.max_frac || 0.6)) + 'px';
  const done = new Set();
  ctx.bind(c.items, (items) => {
    el.replaceChildren();
    (Array.isArray(items) ? items : []).forEach((it, i) => {
      const text = String(it.text ?? '');
      const row = document.createElement('div');
      row.className = 'paste' + (done.has(i) ? ' copied' : '');
      row.dataset.hit = '';

      const head = document.createElement('div');
      head.className = 'head';
      const num = document.createElement('span');
      num.className = 'num';
      num.textContent = String(i + 1);
      const name = document.createElement('div');
      name.className = 'name';
      name.textContent = it.title ?? `Paste ${i + 1}`;
      const size = document.createElement('div');
      size.className = 'size';
      const lines = text.split('\n').length;
      size.textContent = `${text.length.toLocaleString()} characters, ${lines} line${lines === 1 ? '' : 's'}` + (it.note ? ` · ${it.note}` : '');
      name.append(size);

      const btn = document.createElement('button');
      btn.className = 'btn good copy';
      btn.dataset.hit = '';
      const paint = () => {
        btn.replaceChildren(icon(done.has(i) ? 'check' : 'copy', 13));
        const lb = document.createElement('span');
        lb.className = 'lb';
        lb.textContent = done.has(i) ? 'Copied' : 'Copy';
        btn.append(lb);
        btn.setAttribute('aria-label', `Copy ${it.title ?? 'paste ' + (i + 1)}`);
      };
      paint();
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        rippleAt(btn, e);
        // the copy also lands in the agent inbox (no wake), so the session can follow the handover row by row (user, 11-11)
        ctx.act([{ do: 'copy', text }, { do: 'emit', payload: { kind: 'paste_copied', row: i + 1, of: items.length, title: String(it.title ?? '') } }]);
        done.add(i);
        row.classList.add('copied');
        paint();
      });
      head.append(num, name, btn);

      const pre = document.createElement('pre');
      pre.className = 'body';
      pre.textContent = text;
      pre.style.maxHeight = (c.lines || 12) * 1.45 + 'em';
      head.addEventListener('click', () => row.classList.toggle('open'));   // the window follows the new size by itself (bridge.js ResizeObserver)

      row.append(head, pre);
      el.append(row);
      if (!reduced) enter(row, i * 25);
    });
  });
  return el;
});
