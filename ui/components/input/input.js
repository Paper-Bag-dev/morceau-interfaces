import { registerView, html, useLayoutEffect, useRef } from '../../core/view.js';

// {type:"input", placeholder:"Why do you need it?", bind:"/draft", submit:{do:"wake", payload:{answer:"{/draft}"}}}
// Focusing a field asks the host for real keyboard focus (windows are otherwise no-activate); blur gives it back. Preact view: the host
// is a small wrapper, the field inside keeps the `.field` look.
registerView('input', ({ c, ctx, useBind, act }) => {
  const v = useBind({ bind: c.bind });
  const ref = useRef(null);
  useLayoutEffect(() => { const el = ref.current; if (el && document.activeElement !== el) el.value = v ?? ''; }, [v]);
  const key = (e) => {
    if (e.key === 'Enter' && c.submit) { act(c.submit); if (c.clear) { ctx.store.set(c.bind, ''); e.currentTarget.value = ''; } e.currentTarget.blur(); }
    if (e.key === 'Escape') e.currentTarget.blur();
  };
  return html`<input ref=${ref} class="field w-full" type="text" data-hit="" data-nodrag="" spellcheck=${false} placeholder=${c.placeholder || ''}
    onInput=${(e) => ctx.store.set(c.bind, e.currentTarget.value)} onFocus=${() => ctx.send({ t: 'kbd', v: true })}
    onBlur=${() => ctx.send({ t: 'kbd', v: false })} onKeyDown=${key} />`;
}, { cls: 'c-input flex', hit: false });
