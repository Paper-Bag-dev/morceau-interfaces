import { registerView, html, useState } from '../../core/view.js';
import { useNav, useEntities, groupLabel, Dot } from '../nav/nav.js';

// {type:"explorer", title:"Files", entities:{bind:"/entities"}, groups:[{id,label}], by:"group", nav:"@/nav", search:true, only:{kind:"file"}}
// A grouped, searchable list of entities. A click opens the entity through the shared nav model, so every other workbench part follows.
// `by` is the field to group on (default "group"); `only` keeps entities whose fields equal the given values. Groups fold; the selected row lights.
const matches = (e, q) => !q || `${e.title} ${e.sub ?? ''}`.toLowerCase().includes(q);
const keep = (only) => (e) => !only || Object.entries(only).every(([k, v]) => (Array.isArray(v) ? v.includes(e[k]) : e[k] === v));

registerView('explorer', (props) => {
  const { c, useBind } = props;
  const nav = useNav(props);
  const { list } = useEntities(props);
  const title = useBind(c.title);
  const groups = useBind(c.groups);
  const [q, setQ] = useState('');
  const only = useBind(c.only);
  const folded = useBind(c.folded);
  const [shut, setShut] = useState(() => new Set(folded || []));
  const by = c.by || 'group';
  const rows = list.filter(keep(only)).filter((e) => matches(e, q.trim().toLowerCase()));
  const order = (groups || []).map((g) => g.id);
  const byGroup = new Map();
  for (const e of rows) { const k = e[by] ?? ''; if (!byGroup.has(k)) byGroup.set(k, []); byGroup.get(k).push(e); }
  const keys = [...byGroup.keys()].sort((a, b) => (order.indexOf(a) + 1 || 99) - (order.indexOf(b) + 1 || 99));
  const flip = (k) => setShut((s) => { const n = new Set(s); n.has(k) ? n.delete(k) : n.add(k); return n; });

  return html`
    <div class="nav-head"><span class="ttl">${title ?? 'Explorer'}</span><span class="end"><span class="meta">${rows.length} items</span></span></div>
    ${c.search !== false && html`<input class="xp-search" placeholder="Filter" value=${q} onInput=${(e) => setQ(e.currentTarget.value)} data-hit="" />`}
    <div class="xp-list" role="tree">
      ${keys.length === 0 && html`<div class="nav-empty">${q ? `Nothing matches "${q}". Clear the filter to see everything.` : 'No items yet.'}</div>`}
      ${keys.map((k) => {
        const open = q || !shut.has(k);
        const items = byGroup.get(k);
        const bad = items.filter((e) => e.status === 'bad').length;
        return html`<div key=${k} class="xp-group" role="group">
          <button class=${'xp-gh' + (open ? ' open' : '')} data-hit="" onClick=${() => flip(k)} aria-expanded=${open ? 'true' : 'false'}>
            <svg class="chev" viewBox="0 0 12 12" aria-hidden="true"><path d="M4 2.5 7.5 6 4 9.5" /></svg>
            <span class="t">${groupLabel(groups, k) || 'Other'}</span>
            ${bad > 0 && html`<span class="bad">${bad} broken</span>`}
            <span class="n">${items.length}</span>
          </button>
          ${open && html`<div class="xp-rows">${items.map((e) => html`
            <button key=${e.id} class=${'xp-row' + (nav.sel === e.id ? ' sel' : '')} data-hit="" role="treeitem" title=${e.sub ? `${e.title}\n${e.sub}` : e.title}
                aria-selected=${nav.sel === e.id ? 'true' : 'false'} onClick=${() => nav.open(e.id)}>
              <${Dot} status=${e.status} /><span class="t">${e.title}</span>${e.sub && html`<span class="s">${e.sub}</span>`}
            </button>`)}</div>`}
        </div>`;
      })}
    </div>`;
}, { cls: 'c-explorer' });
