import { registerView, html } from '../../core/view.js';
import { useNav, useEntities, groupLabel, Ref, Dot } from '../nav/nav.js';

// {type:"inspector", entities:{bind:"/entities"}, groups, nav:"@/nav", empty:"Pick something to open it here."}
// The editor pane of the workbench: open tabs, back / forward, a breadcrumb, then the selected entity: its note, its sections of references
// (each a chip that opens that entity) and, when it has one, a code peek with line numbers whose references are chips too.
const STR = /("[^"\n]*"|'[^'\n]*'|`[^`\n]*`)/;

/** One source line with string literals tinted; the rest stays plain text (no HTML is ever injected). */
const Code = ({ t }) => html`<code class="ins-code">${String(t).split(STR).map((p, i) => (i % 2 ? html`<span class="str">${p}</span>` : p))}</code>`;

registerView('inspector', (props) => {
  const { c, useBind } = props;
  const nav = useNav(props);
  const { byId } = useEntities(props);
  const groups = useBind(c.groups);
  const e = nav.sel ? byId.get(nav.sel) : null;
  const open = (id) => nav.open(id);

  const tabs = html`<div class="ins-tabs" role="tablist">
    ${nav.tabs.map((id) => {
      const t = byId.get(id);
      return html`<div key=${id} class=${'ins-tab' + (id === nav.sel ? ' on' : '')} role="tab" aria-selected=${id === nav.sel ? 'true' : 'false'}>
        <button class="go" data-hit="" title=${t?.title ?? id} onClick=${() => open(id)}><${Dot} status=${t?.status} /><span class="t">${t?.title ?? id}</span></button>
        <button class="x" data-hit="" title="Close tab" onClick=${() => nav.close(id)}><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3 3l6 6M9 3 3 9" /></svg></button>
      </div>`;
    })}
  </div>`;

  const bar = html`<div class="ins-bar">
    <button class="nb" data-hit="" title="Back" disabled=${!nav.back.length} onClick=${() => nav.back()}><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M7.5 2.5 4 6l3.5 3.5" /></svg></button>
    <button class="nb" data-hit="" title="Forward" disabled=${!nav.fwd.length} onClick=${() => nav.fwd()}><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M4.5 2.5 8 6 4.5 9.5" /></svg></button>
    <span class="crumb">${e ? html`<span class="g">${groupLabel(groups, e.group)}</span><span class="sep">/</span>${e.path ? e.path.split('/').slice(0, -1).map((p) => html`<span class="g">${p}</span><span class="sep">/</span>`) : ''}<span class="here">${e.title}</span>` : ''}</span>
  </div>`;

  if (!e) return html`${tabs}${bar}<div class="nav-empty ins-empty">${useBind(c.empty) ?? 'Pick something in a list to open it here.'}</div>`;

  return html`${nav.tabs.length > 0 && tabs}${bar}
    <div class="ins-body">
      <div class="ins-title"><${Dot} status=${e.status} /><span class="t">${e.title}</span>${e.sub && html`<span class="s">${e.sub}</span>`}</div>
      ${e.note && html`<p class=${'ins-note ' + (e.status || 'ok')}>${e.note}</p>`}
      ${(e.sections || []).map((s) => html`<section key=${s.title} class="ins-sec">
        <div class="h"><span>${s.title}</span><span class="n">${s.refs.length}</span></div>
        <div class="refs">${s.refs.map((id) => html`<${Ref} key=${id} id=${id} e=${byId.get(id)} onOpen=${open} />`)}</div>
      </section>`)}
      ${e.peek?.length > 0 && html`<section class="ins-sec">
        <div class="h"><span>Where in the code</span><span class="n">${e.peek.length} lines</span></div>
        <div class="ins-peek">${e.peek.map((l) => html`<div key=${l.n} class="ln">
          <span class="num">${l.n}</span>
          <div class="src"><${Code} t=${l.t} />${(l.refs || l.links || []).length > 0 && html`<div class="refs">${(l.refs || l.links).map((id) => html`<${Ref} key=${id} id=${id} e=${byId.get(id)} onOpen=${open} dense />`)}</div>`}</div>
        </div>`)}</div>
      </section>`}
    </div>`;
}, { cls: 'c-inspector' });
