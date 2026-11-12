import { registerView, html, Icon, useLayoutEffect, useState } from '../../core/view.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"pastes", items:{bind:"/items"}, lines?:12, max_frac?:0.6}   item: {title, text, note?}
// The list never grows past max_frac of the screen height: past that it scrolls inside the card, so every row and its Copy button stays reachable.
// Paste blocks to hand over in order: one numbered row each with its size and a Copy button. A click on the row shows the exact text
// (scrollable, selectable); Copy puts it on the clipboard (the host does it with pbcopy on macOS, see surface/actions.rs) and the row
// stays marked "Copied", so you can see which ones are done. Preact view.
registerView('pastes', ({ c, useBind, act, host }) => {
  const raw = useBind(c.items);
  const items = Array.isArray(raw) ? raw : [];
  const [done, setDone] = useState({});
  const [open, setOpen] = useState({});
  // screen.availHeight is in CSS px, the same unit as the card; the window itself is sized to its content, so vh would be circular
  host.style.maxHeight = Math.round((screen.availHeight || 900) * (c.max_frac || 0.6)) + 'px';
  useLayoutEffect(() => { if (!reduced) host.querySelectorAll(':scope > .paste').forEach((row, i) => enter(row, i * 25)); }, [JSON.stringify(items)]);
  return items.map((it, i) => {
    const text = String(it.text ?? ''), lines = text.split('\n').length, title = it.title ?? `Paste ${i + 1}`;
    const copy = (e) => {
      e.stopPropagation();
      rippleAt(e.currentTarget, e);
      // the copy also lands in the agent inbox (no wake), so the session can follow the handover row by row (user, 11-11)
      act([{ do: 'copy', text }, { do: 'emit', payload: { kind: 'paste_copied', row: i + 1, of: items.length, title: String(it.title ?? '') } }]);
      setDone({ ...done, [i]: true });
    };
    return html`<div key=${i} class=${'paste' + (done[i] ? ' copied' : '') + (open[i] ? ' open' : '')} data-hit="">
      <div class="head flex items-center gap-2.5" onClick=${() => setOpen({ ...open, [i]: !open[i] })}>
        <span class="num shrink-0">${i + 1}</span>
        <div class="name min-w-0 flex-1">${title}<div class="size">${text.length.toLocaleString()} characters, ${lines} line${lines === 1 ? '' : 's'}${it.note ? ` · ${it.note}` : ''}</div></div>
        <button class="btn good copy shrink-0" data-hit="" aria-label=${`Copy ${title}`} onClick=${copy}><${Icon} name=${done[i] ? 'check' : 'copy'} /><span class="lb">${done[i] ? 'Copied' : 'Copy'}</span></button>
      </div>
      <pre class="body" style=${{ maxHeight: (c.lines || 12) * 1.45 + 'em' }}>${text}</pre>
    </div>`;
  });
}, { cls: 'c-pastes' });
