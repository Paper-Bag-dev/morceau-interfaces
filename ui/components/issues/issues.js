import { registerView, html } from '../../core/view.js';
import { useNav, useEntities, Dot } from '../nav/nav.js';

// {type:"issues", entities:{bind:"/entities"}, nav:"@/nav", levels:[{status:"bad", label:"Broken"}, {status:"warn", label:"Check"}], title:"Problems"}
// Every entity whose status is not ok, grouped by level, worst first, with its note as the reason. A click opens it through nav.
const LEVELS = [{ status: 'bad', label: 'Broken' }, { status: 'warn', label: 'Check' }];

registerView('issues', (props) => {
  const { c, useBind } = props;
  const nav = useNav(props);
  const { list } = useEntities(props);
  const levels = useBind(c.levels) || LEVELS;
  const total = list.filter((e) => levels.some((l) => l.status === e.status)).length;
  return html`
    <div class="nav-head"><span class="ttl">${useBind(c.title) ?? 'Problems'}</span><span class="end"><span class="meta">${total} found</span></span></div>
    ${total === 0 && html`<div class="nav-empty">Nothing broken. Every item has both ends.</div>`}
    <div class="is-list">
      ${levels.map((l) => {
        const items = list.filter((e) => e.status === l.status);
        if (!items.length) return null;
        return html`<section key=${l.status} class="is-level">
          <div class=${'is-lh ' + l.status}><${Dot} status=${l.status} /><span>${l.label}</span><span class="n">${items.length}</span></div>
          ${items.map((e) => html`<button key=${e.id} class=${'is-row' + (nav.sel === e.id ? ' sel' : '')} data-hit="" onClick=${() => nav.open(e.id)}>
            <span class="t">${e.title}${e.sub && html`<span class="s">${e.sub}</span>`}</span>
            ${e.note && html`<span class="why">${e.note}</span>`}
          </button>`)}
        </section>`;
      })}
    </div>`;
}, { cls: 'c-issues' });
