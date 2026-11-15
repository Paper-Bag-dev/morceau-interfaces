import { registerView, html } from '../../core/view.js';
import { useNav, useEntities, Dot } from '../nav/nav.js';

// {type:"steps", entities:{bind:"/entities"}, route:{bind:"/route"}, nav:"@/nav", title:"Gym tick, end to end"}
// An ordered route through entities (a user flow, a request path). The step that is open lights; previous / next walk the route through nav,
// so the inspector and the wire follow each step. Steps whose entity is not ok show their dot, so a break on the route stands out.
registerView('steps', (props) => {
  const { c, useBind } = props;
  const nav = useNav(props);
  const { byId } = useEntities(props);
  const route = useBind(c.route) || [];
  const at = route.indexOf(nav.sel);
  const go = (i) => route[i] && nav.open(route[i]);
  return html`
    <div class="nav-head"><span class="ttl">${useBind(c.title) ?? 'Route'}</span>
      <span class="st-nav">
        <button class="nb" data-hit="" title="Previous step" disabled=${at <= 0} onClick=${() => go(at - 1)}><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M7.5 2.5 4 6l3.5 3.5" /></svg></button>
        <span class="meta">${at >= 0 ? `${at + 1} of ${route.length}` : `${route.length} steps`}</span>
        <button class="nb" data-hit="" title="Next step" disabled=${at >= route.length - 1} onClick=${() => go(at < 0 ? 0 : at + 1)}><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M4.5 2.5 8 6 4.5 9.5" /></svg></button>
      </span></div>
    <ol class="st-list">
      ${route.map((id, i) => {
        const e = byId.get(id);
        return html`<li key=${id} class=${'st-step' + (i === at ? ' on' : '') + (at >= 0 && i < at ? ' past' : '')}>
          <button data-hit="" onClick=${() => go(i)}><span class="pin"></span><span class="t">${e?.title ?? id}</span>${e?.sub && html`<span class="s">${e.sub}</span>`}${e && e.status !== 'ok' && html`<${Dot} status=${e.status} />`}</button>
        </li>`;
      })}
    </ol>`;
}, { cls: 'c-steps' });
