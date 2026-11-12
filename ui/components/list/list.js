import { registerView, html, useLayoutEffect, useRef } from '../../core/view.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"list", items:{bind:"/todos"}, max:6}   item: {text, sub?, badge?, tip?, action?}   sizes: ui/tokens/list.json. Preact view.
registerView('list', ({ c, useBind, act, host }) => {
  const items = (Array.isArray(useBind(c.items)) ? useBind(c.items) : []).slice(0, c.max || 8);
  const first = useRef(true);
  useLayoutEffect(() => {                                // rows fade in one after another, also on first show
    if (!reduced) [...host.children].forEach((row, i) => enter(row, (first.current ? 60 : 0) + i * 35));
    first.current = false;
  }, [JSON.stringify(items)]);
  return items.map((it, i) => html`
    <div key=${i} class=${'item flex items-center gap-2' + (it.action ? ' act' : '')} data-hit=""
      title=${it.tip ?? [it.text, it.sub].filter(Boolean).join('\n')}
      onClick=${it.action ? (e) => { rippleAt(e.currentTarget, e); act(it.action); } : undefined}>
      <div class="txt min-w-0 flex-1"><div class="ttl truncate">${it.text ?? ''}</div>${it.sub ? html`<div class="sub truncate">${it.sub}</div>` : null}</div>
      ${it.badge ? html`<span class="badge shrink-0">${it.badge}</span>` : null}
    </div>`);
}, { cls: 'c-list', hit: false });
