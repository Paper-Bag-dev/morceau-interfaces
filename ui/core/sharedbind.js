// The shared store as a card sees it: {get, set, sub} over pointers starting with "@" ({"bind": "@/sel"}, {"do": "set", "path": "@/sel"}).
// Linked cards (docs/process-model.md) all reach the root page's window.opener.morcShared: one object, every card sees a change at once.
// Unlinked cards (Windows, MORC_LINKED=0) share through the HOST instead: a write goes to the host ({"t":"shared"}), which keeps the
// state and passes it to every other card; a new card starts from the host's copy (init.shared). Same API either way.

import { getIn, setIn } from './pointer.js';
import { send } from './bridge.js';

let local = { state: {}, subs: new Set() };
function apply(p, v) { const prev = local.state; local.state = setIn(prev, p, v); local.subs.forEach((l) => l(local.state, prev)); }
const fallback = {
  get: (p) => getIn(local.state, p),
  set: (p, v) => { apply(p, v); send({ t: 'shared', path: p, value: v }); },
  subscribe: (l) => { local.subs.add(l); return () => local.subs.delete(l); },
};
/** The host relayed another card's write (or `morc patch @shared`). */
export function applyRemote(p, v) { apply(p, v); }
/** A new card: start from the host's copy of the shared state. */
export function seedShared(state) { if (state && typeof state === 'object') { const prev = local.state; local.state = state; local.subs.forEach((l) => l(local.state, prev)); } }

export function shared() {
  let S = null;
  try { S = window.opener && window.opener.morcShared; } catch (_) { S = null; }
  const s = S || fallback;
  return {
    linked: !!S,
    get: (p) => s.get(p),
    set: (p, v) => s.set(p, v),
    keys: () => Object.keys((S ? S.getState() : local.state) || {}),
    // The listener lives in the root page; if this card is gone the call throws and the listener removes itself.
    sub(p, cb) {
      let off = null;
      off = s.subscribe((next, prev) => {
        const a = getIn(next, p), b = getIn(prev, p);
        if (a === b) return;
        try { cb(a); } catch (_) { if (off) off(); }
      });
      return off;
    },
  };
}
