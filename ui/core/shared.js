// The root page (root.html): opens every card on the host's request and holds the state cards share.
// A card reaches it as window.opener.morcShared (same web process, so it is the same object, not a copy).

// A small store with zustand's vanilla API: getState, setState(partial | fn, replace), subscribe(listener(state, prev)) -> unsubscribe.
export function createStore(init) {
  let state;
  const listeners = new Set();
  const getState = () => state;
  const setState = (partial, replace) => {
    const next = typeof partial === 'function' ? partial(state) : partial;
    if (Object.is(next, state)) return;
    const prev = state;
    state = replace || typeof next !== 'object' || next === null ? next : Object.assign({}, state, next);
    listeners.forEach((l) => l(state, prev));
  };
  const subscribe = (l) => { listeners.add(l); return () => listeners.delete(l); };
  const api = { getState, setState, subscribe };
  state = init(setState, getState, api);
  return api;
}

import { getIn, setIn } from './pointer.js';

window.morcShared = createStore(() => ({}));
// Pointer helpers on the shared store, for cards ({"bind": "@/path"}, "shared.set") and the host (`morc patch @shared /path value`).
window.morcShared.get = (path) => getIn(window.morcShared.getState(), path);
window.morcShared.set = (path, value) => window.morcShared.setState((s) => setIn(s, path, value), true);
window.morcSharedPatch = (map) => { for (const [p, v] of Object.entries(map || {})) window.morcShared.set(p, v); };

// The host asks for card <key>; wry builds its web view into the window the host made for it.
window.morcOpen = (key) => window.open('index.html?k=' + key, 'k' + key);

window.ipc.postMessage('root:ready');
