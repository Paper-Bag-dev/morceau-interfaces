// Builds the DOM from a flat component list and keeps it bound to the data model.
//   components: [{id, type, ...props, children?: [ids]}]   root = id "root" (or the first)
//   any prop can be a literal, {bind: "/path"}, or a string template "CPU {/cpu:0}%"

import { Store } from './store.js';
import { get } from './registry.js';
import { enter, leave, reduced } from './motion.js';
import { measureSoon } from './bridge.js';

const TPL = /\{(\/[^}:]*)(?::(\d))?\}/g;

function fmt(v, digits) {
  if (v == null) return '';
  if (typeof v === 'number') return digits != null ? v.toFixed(+digits) : Number.isInteger(v) ? String(v) : v.toFixed(1);
  return typeof v === 'object' ? JSON.stringify(v) : String(v);
}

export class Renderer {
  constructor(root, send) {
    this.root = root;
    this.send = send;
    this.store = new Store({});
    this.comps = new Map();
    this.unsubs = [];
  }

  init(msg) {
    this.store = new Store(msg.data || {});
    this.comps = new Map((msg.components || []).map((c) => [c.id, c]));
    this.build();
  }

  patch(msg) {
    for (const [p, v] of Object.entries(msg.data || {})) this.store.set(p, v);
    if (msg.components && msg.components.length) {
      if (msg.attach && msg.attach.parent) return this.attach(msg.attach, msg.components);   // piece-by-piece build (morc stream)
      for (const c of msg.components) this.comps.set(c.id, c);
      this.build();
    }
  }

  /**
   * Mount ONE new component into a parent that is already on screen, with an enter animation, leaving everything else untouched
   * (stores, running animations, charts). at = {parent, before?}; comps[0] is the new node, the rest are its descendants' specs.
   */
  attach(at, comps) {
    for (const c of comps) this.comps.set(c.id, c);
    const parent = this.comps.get(at.parent), top = comps[0];
    if (parent && at.replace) {                       // swap a skeleton for the real component, same slot
      const kids = parent.children || (parent.children = []);
      const i = kids.indexOf(at.replace);
      if (i >= 0) kids[i] = top.id; else kids.push(top.id);
      this.comps.delete(at.replace);
    } else if (parent) {
      const kids = parent.children || (parent.children = []);
      if (!kids.includes(top.id)) { const i = at.before ? kids.indexOf(at.before) : -1; if (i >= 0) kids.splice(i, 0, top.id); else kids.push(top.id); }
    }
    const pel = this.root.querySelector(`[data-cid="${CSS.escape(at.parent)}"]`);
    if (!pel) return this.build();
    const el = this.node(top.id);
    if (at.replace) {
      const old = pel.querySelector(`:scope > [data-cid="${CSS.escape(at.replace)}"]`);
      if (old) old.replaceWith(el); else pel.append(el);
      if (!reduced) enter(el);
      measureSoon();
      return;
    }
    const ref = at.before ? pel.querySelector(`:scope > [data-cid="${CSS.escape(at.before)}"]`) : null;
    pel.insertBefore(el, ref);
    if (!reduced) enter(el);
    measureSoon();
  }

  build() {
    this.unsubs.forEach((u) => u());
    this.unsubs = [];
    const rootId = this.comps.has('root') ? 'root' : [...this.comps.keys()][0];
    const rootEl = rootId ? this.node(rootId) : document.createElement('div');
    // Every card sits on ONE plate by default (the frosted material of ui/components/plate), so its pieces never float loose on a busy page.
    // Only a layout root gets it (a lone text or stat already is a plate); "plate": false on the root opts out.
    const rc = rootId && this.comps.get(rootId);
    if (rc && rc.plate !== false && ['column', 'row', 'panel'].includes(rc.type)) { rootEl.classList.add('card-plate'); rootEl.dataset.hit = ''; }
    this.root.replaceChildren(rootEl);
  }

  node(id) {
    const c = this.comps.get(id);
    if (!c) return this.error(`missing component "${id}"`);
    const fn = get(c.type);
    if (!fn) return this.error(`unknown type "${c.type}"`);
    try {
      const el = fn(c, this.ctx(c));
      el.dataset.cid = id;
      if (c.when) this.applyWhen(el, c.when);
      return el;
    } catch (e) {
      return this.error(`${c.type} "${id}": ${e.message}`);
    }
  }

  /**
   * Conditional visibility: {when: {path: "/answered", eq: false}}  (also ne, truthy, falsy).
   * This is how a click transforms the surface: the question hides, the result shows. Changes animate and the window re-fits.
   */
  applyWhen(el, w) {
    const test = (v) => {
      if ('eq' in w) return v === w.eq;
      if ('ne' in w) return v !== w.ne;
      if (w.truthy) return !!v;
      if (w.falsy) return !v;
      return !!v;
    };
    let shown = null;
    const apply = (v) => {
      const now = test(v);
      if (now === shown) return;
      const first = shown === null;
      shown = now;
      if (now) {
        el.style.display = '';
        if (!first && !reduced) enter(el);
      } else if (first || reduced) {
        el.style.display = 'none';
      } else {
        leave(el).finished.then(() => { if (!shown) el.style.display = 'none'; measureSoon(); }, () => { el.style.display = 'none'; });
        return;
      }
      measureSoon();
    };
    apply(this.store.get(w.path));
    this.unsubs.push(this.store.sub(w.path, apply));
  }

  error(msg) {
    this.send({ t: 'log', msg });
    const el = document.createElement('div');
    el.className = 'c-text sm tone-bad';
    el.dataset.hit = '';
    el.textContent = '⚠ ' + msg;
    return el;
  }

  /** Resolve {/path} templates inside any JSON value (used for action payloads). */
  resolve(v) {
    if (typeof v === 'string') return v.replace(TPL, (_, p, d) => fmt(this.store.get(p), d));
    if (Array.isArray(v)) return v.map((x) => this.resolve(x));
    if (v && typeof v === 'object') return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, this.resolve(x)]));
    return v;
  }

  ctx(c) {
    const self = this;
    return {
      store: this.store,
      child: (id) => this.node(id),
      children: (c.children || []).map((id) => () => this.node(id)),
      send: this.send,
      /** Call cb(value) now and whenever the value changes. Accepts literal, {bind}, or "template {/x}". */
      bind(v, cb) {
        if (v && typeof v === 'object' && 'bind' in v) {
          cb(self.store.get(v.bind));
          self.unsubs.push(self.store.sub(v.bind, cb));
        } else if (typeof v === 'string' && TPL.test(v)) {
          TPL.lastIndex = 0;
          const paths = [...v.matchAll(TPL)].map((m) => m[1]);
          const run = () => cb(self.resolve(v));
          run();
          for (const p of new Set(paths)) self.unsubs.push(self.store.sub(p, run));
        } else {
          TPL.lastIndex = 0;
          cb(v);
        }
      },
      /** Run an action (or a list). Page-local ones are handled here; the rest go to the host. */
      act(action) {
        if (!action) return;
        if (Array.isArray(action)) return action.forEach((a) => self.act(a));
        const a = self.resolve(action);
        switch (a.do) {
          case 'set': return self.store.set(a.path, a.value);
          case 'toggle': return self.store.set(a.path, !self.store.get(a.path));
          // the browser clipboard is tried, AND the host copies too (pbcopy on macOS, where WKWebView may refuse a morc:// page; surface/actions.rs)
          case 'copy': if (navigator.clipboard) navigator.clipboard.writeText(String(a.text ?? '')).catch(() => {}); return self.send({ t: 'act', action: a, comp: c.id });
          case 'close': return self.send({ t: 'close' });
          default: return self.send({ t: 'act', action: a, comp: c.id });
        }
      },
    };
  }
}

Renderer.prototype.act = function (action) { return this.ctx({ id: '' }).act(action); };
