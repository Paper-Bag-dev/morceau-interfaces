// Reactive data model addressed by JSON pointers ("/cpu", "/items/0/text").
// Components subscribe to a path; a patch to "/a" notifies subscribers of "/a", "/a/b" and the parent "/".

const unesc = (s) => s.replace(/~1/g, '/').replace(/~0/g, '~');
const parts = (p) => (p === '' || p === '/' ? [] : p.split('/').slice(1).map(unesc));

export class Store {
  constructor(data = {}) {
    this.data = data;
    this.subs = new Set(); // {path, cb}
  }

  get(path) {
    let o = this.data;
    for (const k of parts(path)) {
      if (o == null) return undefined;
      o = o[k];
    }
    return o;
  }

  set(path, value) {
    const ks = parts(path);
    if (!ks.length) { this.data = value; return this.notify('/'); }
    let o = this.data;
    for (let i = 0; i < ks.length - 1; i++) {
      if (o[ks[i]] == null || typeof o[ks[i]] !== 'object') o[ks[i]] = /^\d+$/.test(ks[i + 1]) ? [] : {};
      o = o[ks[i]];
    }
    o[ks[ks.length - 1]] = value;
    this.notify(path);
  }

  sub(path, cb) {
    const s = { path, cb };
    this.subs.add(s);
    return () => this.subs.delete(s);
  }

  notify(path) {
    if (this.onChange) this.onChange(path);   // persisted cards: core/main.js schedules a snapshot
    for (const s of [...this.subs]) {
      const a = s.path === '' ? '/' : s.path;
      if (path === '/' || a === '/' || a === path || a.startsWith(path + '/') || path.startsWith(a + '/')) s.cb(this.get(s.path));
    }
  }
}
