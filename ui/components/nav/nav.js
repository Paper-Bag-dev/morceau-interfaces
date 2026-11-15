// nav: the one navigation model the workbench parts share (explorer, inspector, wire, issues, steps). No UI of its own.
//
// Entities (any source): {id, title, sub?, group?, kind?, status?: "ok"|"warn"|"bad", note?, sections?: [{title, refs: [id]}],
//                         from?: [{ref, n?}], to?: [{ref, n?}], peek?: [{n, t, refs?: [id]}]}
// Nav state, kept at a shared-store path (default "@/nav", so every card bound to it moves together; a second app uses another path):
//   {sel: id|null, tabs: [id], back: [id], fwd: [id]}
import { html, useRef } from '../../core/view.js';

const EMPTY = { sel: null, tabs: [], back: [], fwd: [] };
const MAX_TABS = 8, MAX_HIST = 40;

/** The nav state at c.nav (default "@/nav") and the moves on it. Every move writes the whole object, so readers never see half an update. */
export function useNav({ c, useBind, act }) {
  const path = c.nav || '@/nav';
  const s = { ...EMPTY, ...(useBind({ bind: path }) || {}) };
  const write = (next) => act({ do: 'set', path, value: next });
  const withTab = (tabs, id) => (tabs.includes(id) ? tabs : [...tabs, id].slice(-MAX_TABS));
  return {
    ...s,
    open(id) {
      if (!id || id === s.sel) return;
      write({ sel: id, tabs: withTab(s.tabs, id), back: s.sel ? [...s.back, s.sel].slice(-MAX_HIST) : s.back, fwd: [] });
    },
    close(id) {
      const tabs = s.tabs.filter((t) => t !== id);
      write({ ...s, tabs, sel: s.sel === id ? tabs[tabs.length - 1] ?? null : s.sel });
    },
    back() {
      if (!s.back.length) return;
      const prev = s.back[s.back.length - 1];
      write({ sel: prev, tabs: withTab(s.tabs, prev), back: s.back.slice(0, -1), fwd: s.sel ? [s.sel, ...s.fwd] : s.fwd });
    },
    fwd() {
      if (!s.fwd.length) return;
      const next = s.fwd[0];
      write({ sel: next, tabs: withTab(s.tabs, next), back: s.sel ? [...s.back, s.sel] : s.back, fwd: s.fwd.slice(1) });
    },
  };
}

/** The entities at c.entities (list or {bind}), indexed by id once per change. */
export function useEntities({ c, useBind }) {
  const list = useBind(c.entities) || [];
  const memo = useRef({ list: null, byId: new Map() });
  if (memo.current.list !== list) memo.current = { list, byId: new Map(list.map((e) => [e.id, e])) };
  return { list, byId: memo.current.byId };
}

/** Group labels from c.groups ([{id, label}]) with the entity's own group id as the fallback. */
export const groupLabel = (groups, id) => (groups || []).find((g) => g.id === id)?.label ?? id ?? '';

/** A clickable reference to an entity: status dot + title (+ an optional line number). Unknown ids still render, muted. */
export function Ref({ e, id, n, onOpen, dense = false }) {
  const title = e?.title ?? id;
  return html`<button class=${'nav-ref' + (dense ? ' dense' : '') + (e ? '' : ' missing')} data-hit="" title=${e?.sub ? `${title}\n${e.sub}` : title}
      onClick=${(ev) => { ev.stopPropagation(); onOpen(id); }}>
    <i class=${'nav-dot ' + (e?.status || 'ok')}></i><span class="t">${title}</span>${n ? html`<span class="n">:${n}</span>` : ''}
  </button>`;
}

/** The status dot alone, for rows that draw their own text. */
export const Dot = ({ status }) => html`<i class=${'nav-dot ' + (status || 'ok')}></i>`;
