// JSON pointers over plain state ("/items/0/text"), shared by the card store's readers and the shared store (root page).
// setIn is immutable (copies only along the path), so a zustand-style subscriber can compare prev and next by identity.

const unesc = (s) => s.replace(/~1/g, '/').replace(/~0/g, '~');
export const parts = (p) => (p === '' || p === '/' ? [] : p.split('/').slice(1).map(unesc));

export function getIn(o, path) {
  for (const k of parts(path)) {
    if (o == null) return undefined;
    o = o[k];
  }
  return o;
}

export function setIn(o, path, value) {
  const ks = parts(path);
  const go = (cur, i) => {
    if (i === ks.length) return value;
    const k = ks[i];
    const base = cur != null && typeof cur === 'object' ? cur : /^\d+$/.test(k) ? [] : {};
    const copy = Array.isArray(base) ? base.slice() : { ...base };
    copy[k] = go(base[k], i + 1);
    return copy;
  };
  return go(o, 0);
}
