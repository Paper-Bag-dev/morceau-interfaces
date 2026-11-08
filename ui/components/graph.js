import { register } from '../core/registry.js';
import { reduced } from '../core/motion.js';
import { accent } from '../charts/util.js';
import * as literacy from '../core/literacy.js';
import { working } from '../core/working.js';
import { EXPLAIN } from '../charts/explain.js';

// {type:"graph", data:{bind:"/graph"}, width:460, height:320}
// A node-link diagram on Cytoscape.js (vendored in ui/vendor, loaded only when a graph is shown). Schema: docs/graphs.md:
//   {nodes:[{id,label,group,size}], edges:[{source,target,weight,kind}], flow:[ids in order], groups:[{id,label}]}
// Layout is an architecture diagram, not a hairball: each group is a labelled layer box, layers sit in a grid, nodes are packed inside
// their layer (route nodes first, then biggest first). Everything stays draggable; the background pans; the wheel zooms; double-click
// re-fits. Links are faint until you hover a node, which lights its neighbourhood. The orange dashed `flow` route is the one thing
// allowed to move on its own. NB: Cytoscape ignores alpha in rgba(); opacity lives in the *-opacity style properties.

let libPromise = null;
function loadLib() {
  if (window.cytoscape) return Promise.resolve(window.cytoscape);
  libPromise ||= new Promise((ok, fail) => {
    const s = document.createElement('script');
    s.src = 'vendor/cytoscape.min.js';
    s.onload = () => ok(window.cytoscape);
    s.onerror = () => fail(new Error('could not load vendor/cytoscape.min.js'));
    document.head.append(s);
  });
  return libPromise;
}

const hslToRgb = (h, s, l) => {
  s /= 100; l /= 100;
  const k = (n) => (n + h / 30) % 12, a = s * Math.min(l, 1 - l), f = (n) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return `rgb(${[f(0), f(8), f(4)].map((v) => Math.round(255 * v)).join(',')})`;
};
const hsl = (h, s, l) => hslToRgb(((h % 360) + 360) % 360, s, l);

function colorsNow() {
  const cs = getComputedStyle(document.getElementById('stage'));
  const t = (name, fb) => (cs.getPropertyValue(name).trim() || fb).replace(/%/g, '').split(/\s+/).map(Number);
  const [h, s, l] = t('--foreground', '0 0% 98%');
  return { accent: accent(), fg: { h, s, l } };
}

/** Deterministic grid of layers, nodes packed inside each (route nodes first). Returns Map(id -> {x,y}) in canvas px. */
function arrange(groups, nodes, flow, W, H) {
  const L = groups.length, cols = Math.max(1, Math.ceil(Math.sqrt(L * (W / H)))), rows = Math.ceil(L / cols);
  const px = 12, py = 10, cw = (W - px * 2) / cols, ch = (H - py * 2) / rows;
  const fi = new Map(flow.map((id, i) => [id, i]));
  const rank = (n) => (fi.has(n.id) ? fi.get(n.id) : 1000);
  const pos = new Map();
  groups.forEach((gr, gi) => {
    const x0 = px + (gi % cols) * cw, y0 = py + Math.floor(gi / cols) * ch;
    const mine = nodes.filter((n) => (n.group || 'nodes') === gr.id).sort((a, b) => rank(a) - rank(b) || (b.size || 1) - (a.size || 1));
    const iw = cw - 26, ih = ch - 44, n = Math.max(1, mine.length);
    const c = Math.max(1, Math.min(n, Math.ceil(Math.sqrt((n * iw) / ih)))), r = Math.ceil(n / c);
    mine.forEach((nd, i) => pos.set(nd.id, { x: x0 + 13 + ((i % c) + 0.5) * (iw / c), y: y0 + 30 + (Math.floor(i / c) + 0.5) * (ih / r) }));
  });
  return pos;
}

register('graph', (c, ctx) => {
  const W = c.width || 460, H = c.height || 320;
  const el = document.createElement('div');
  el.className = 'c-graph';
  el.dataset.hit = '';
  const frame = document.createElement('div');
  frame.className = 'gframe';
  frame.style.width = W + 'px';
  frame.style.height = H + 'px';
  const inner = document.createElement('div');               // counter-zoomed so Cytoscape sees ordinary 1:1 pointer coordinates
  inner.className = 'gcanvas';
  frame.append(inner);
  const HINT = 'Drag a dot, scroll to zoom, double-click to fit.';
  const info = document.createElement('div');
  info.className = 'ginfo plate';
  info.textContent = HINT;
  el.append(frame, info);

  const fit = () => {
    const z = window.__zoom || 1;
    inner.style.zoom = String(1 / z);
    inner.style.width = W * z + 'px';
    inner.style.height = H * z + 'px';
  };
  fit();

  let cy = null, raf = 0, data = null, started = false;
  const busy = working(el);                                  // blue pulse until the diagram is drawn

  // one-time explainer, same loop as the unfamiliar chart kinds
  function caption() {
    const lit = literacy.get('graph');
    if (lit.got > 0 || lit.seen >= 2 || !EXPLAIN.graph) return;
    const cap = document.createElement('div');
    cap.className = 'explain';
    const txt = document.createElement('div');
    txt.className = 'txt';
    txt.textContent = EXPLAIN.graph.short;
    const chips = document.createElement('div');
    chips.className = 'chips';
    const mk = (t, fn) => { const b = document.createElement('button'); b.className = 'mini'; b.textContent = t; b.addEventListener('click', fn); return b; };
    const done = () => cap.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 160 }).finished.then(() => cap.remove(), () => cap.remove());
    chips.append(mk('Got it', () => { literacy.mark('graph', 'got'); done(); }), mk('Still confusing', () => { literacy.mark('graph', 'confused'); txt.textContent = EXPLAIN.graph.plain; chips.replaceChildren(mk('OK', done)); }));
    cap.append(txt, chips);
    el.append(cap);
    literacy.mark('graph', 'seen');
  }

  function build(Cy, g) {
    const { accent: ac, fg } = colorsNow();
    const groups = g.groups && g.groups.length ? g.groups : [...new Set(g.nodes.map((n) => n.group || 'nodes'))].map((id) => ({ id, label: id }));
    const gi = new Map(groups.map((x, i) => [x.id, i]));
    const hue = (id) => ac.h + ((gi.get(id) ?? 0) / Math.max(1, groups.length - 1)) * 140;   // a family walking round from the accent
    const flow = g.flow || [], onflow = new Set(flow);
    const ids = new Set(g.nodes.map((n) => n.id));
    const pos = arrange(groups, g.nodes, flow, W, H);

    const els = [];
    for (const gr of groups) els.push({ data: { id: 'layer:' + gr.id, label: gr.label || gr.id, hue: hue(gr.id) }, classes: 'layer' });
    for (const n of g.nodes) els.push({ data: { id: n.id, label: (n.label || n.id).replace(/\.(rs|js|py)$/, ''), parent: 'layer:' + (n.group || 'nodes'), hue: hue(n.group), size: n.size || 1 }, position: pos.get(n.id), classes: onflow.has(n.id) ? 'onflow' : '' });
    g.edges.forEach((e, i) => { if (ids.has(e.source) && ids.has(e.target)) els.push({ data: { id: 'e' + i, source: e.source, target: e.target, weight: e.weight || 1 } }); });
    for (let i = 1; i < flow.length; i++) els.push({ data: { id: 'f' + i, source: flow[i - 1], target: flow[i], weight: 1 }, classes: 'flow' });

    const A = hsl(ac.h, ac.s, ac.l), F = hsl(fg.h, fg.s, fg.l);
    const body = '"Segoe UI Variable Text","Segoe UI",sans-serif', display = '"Bahnschrift","Segoe UI",sans-serif';
    cy = Cy({
      container: inner, elements: els, layout: { name: 'preset' }, pixelRatio: window.devicePixelRatio || 1, minZoom: 0.35, maxZoom: 3.2, wheelSensitivity: 0.25,
      boxSelectionEnabled: false, selectionType: 'single',
      style: [
        { selector: 'node.layer', style: { shape: 'round-rectangle', 'background-color': (n) => hsl(n.data('hue'), 70, 55), 'background-opacity': 0.09, 'border-width': 1, 'border-color': (n) => hsl(n.data('hue'), 70, 55), 'border-opacity': 0.45,
          padding: 14, label: 'data(label)', 'text-valign': 'top', 'text-halign': 'left', 'text-margin-x': 9, 'text-margin-y': 12, color: F, 'text-opacity': 0.7, 'font-family': display, 'font-size': 10.5, 'font-weight': 600, events: 'no' } },
        { selector: 'node:childless', style: { shape: 'ellipse', width: (n) => 8 + Math.sqrt(n.data('size')) * 2.4, height: (n) => 8 + Math.sqrt(n.data('size')) * 2.4,
          'background-color': (n) => hsl(n.data('hue'), 72, 56), 'border-width': 1.4, 'border-color': F, 'border-opacity': 0.85, label: '', 'font-family': body, 'font-size': 8.5, color: F,
          'text-valign': 'bottom', 'text-margin-y': 3, 'text-background-color': hsl(fg.h, fg.s, fg.l > 50 ? 12 : 96), 'text-background-opacity': 0.9, 'text-background-padding': 2, 'text-background-shape': 'roundrectangle', 'min-zoomed-font-size': 5 } },
        { selector: 'node.onflow', style: { 'border-color': A, 'border-opacity': 1, 'border-width': 2.6, label: 'data(label)', 'z-index': 30 } },
        { selector: 'edge', style: { 'curve-style': 'bezier', width: (e) => 0.7 + Math.min(2, Math.sqrt(e.data('weight')) * 0.4), 'line-color': F, 'line-opacity': 0.13, 'target-arrow-color': F, 'target-arrow-shape': 'triangle', 'arrow-scale': 0.5 } },
        { selector: 'edge.flow', style: { width: 2.6, 'line-color': A, 'line-opacity': 1, 'target-arrow-color': A, 'target-arrow-shape': 'triangle', 'arrow-scale': 0.95, 'line-style': 'dashed', 'line-dash-pattern': [7, 5], 'z-index': 25 } },
        { selector: '.dim', style: { opacity: 0.12 } },
        { selector: 'node.hl', style: { label: 'data(label)', opacity: 1, 'border-color': A, 'border-opacity': 1, 'border-width': 2.4, 'z-index': 40 } },
        { selector: 'edge.hl', style: { 'line-color': A, 'line-opacity': 0.85, 'target-arrow-color': A, opacity: 1, width: 1.8 } },
      ],
    });
    cy.fit(undefined, 8);
    busy.done();

    // arrive: layers fade up in turn, then the links (answers the surface appearing; no looping decoration)
    if (!reduced) {
      cy.elements().style('opacity', 0);
      groups.forEach((gr, i) => setTimeout(() => cy && cy.getElementById('layer:' + gr.id).add(cy.nodes(`[parent = "layer:${gr.id}"]`)).animate({ style: { opacity: 1 } }, { duration: 360 }), i * 110));
      setTimeout(() => cy && cy.edges().animate({ style: { opacity: 1 } }, { duration: 320 }), groups.length * 110 + 120);
    }

    // hover lights the neighbourhood (a node's links are faint otherwise)
    cy.on('mouseover', 'node:childless', (ev) => {
      const n = ev.target, hood = n.closedNeighborhood();
      cy.elements().not(hood).not('.layer').addClass('dim');
      hood.addClass('hl').removeClass('dim');
      info.textContent = `${n.id()}   uses ${n.outgoers('node:childless').length}, used by ${n.incomers('node:childless').length}`;
    });
    cy.on('mouseout', 'node:childless', () => { cy.elements().removeClass('dim hl'); info.textContent = HINT; });
    cy.on('dbltap', (ev) => { if (ev.target === cy) cy.animate({ fit: { padding: 8 } }, { duration: 300 }); });

    // the route: marching dashes (the one self-moving thing); stops when the surface goes away
    if (!reduced && flow.length > 1) {
      let off = 0;
      const tick = () => { if (!el.isConnected || !cy) return; off = (off - 0.35) % 12; cy.edges('.flow').style('line-dash-offset', off); raf = requestAnimationFrame(tick); };
      raf = requestAnimationFrame(tick);
    }
  }

  async function render() {
    if (!data || !started) return;
    try {
      const Cy = await loadLib();
      if (cy) { cy.destroy(); cy = null; cancelAnimationFrame(raf); }
      fit();
      build(Cy, data);
    } catch (e) { info.textContent = String(e.message || e); ctx.send({ t: 'log', msg: 'graph: ' + (e.message || e) }); }
  }

  ctx.bind(c.data, (v) => {
    data = v && Array.isArray(v.nodes) && v.nodes.length ? v : null;
    if (!data) return;
    if (!started && !document.getElementById('stage').dataset.entered) { addEventListener('morc-enter', () => { started = true; caption(); render(); }, { once: true }); return; }
    if (!started) { started = true; caption(); }
    render();
  });
  addEventListener('morc-zoom', function onZoom() { if (!el.isConnected) return removeEventListener('morc-zoom', onZoom); fit(); if (cy) { cy.resize(); cy.fit(undefined, 8); } });   // the surface auto-fits to the screen: re-fit the diagram with it
  return el;
});
