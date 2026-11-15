import { registerView, html, useState, useEffect, useRef } from '../../core/view.js';
import { useNav, useEntities } from '../nav/nav.js';

// {type:"flowmap", entities:{bind:"/entities"}, groups, nav:"@/nav", title:"Flow"}
// The whole entity graph on React Flow (vendored bundle, ui/vendor/reactflow): pan, scroll, zoom, minimap, drag any node. dagre lays it out
// left to right (who sends -> what -> who reads, from each entity's `from` / `to`). Click a node to open it in every workbench card; the open one
// lights up, its links animate and the view glides to it. React runs in its own root inside this Preact host; the two never share a tree.
let libP = null;
function loadLib() {
  if (!libP) {
    const css = document.createElement('link');
    css.rel = 'stylesheet'; css.href = 'vendor/reactflow/reactflow.css';
    document.head.append(css);
    libP = import('../../vendor/reactflow/reactflow.bundle.js?v=7ef5d38c');   // the hash pins the vendored build (and busts a cached old one)
  }
  return libP;
}

const COL = { bad: '#f0705e', warn: '#e9b949' };
const width = (t) => Math.min(220, 26 + String(t).length * 7.2);

/** Nodes and edges once per entity list: every link a message makes (sender -> message -> reader), laid out by dagre. */
function graphOf(L, list) {
  const g = new L.dagre.graphlib.Graph();
  g.setGraph({ rankdir: 'LR', nodesep: 10, ranksep: 90, marginx: 20, marginy: 20 });
  g.setDefaultEdgeLabel(() => ({}));
  const ids = new Set(list.map((e) => e.id)), edges = [];
  for (const e of list) g.setNode(e.id, { width: width(e.title), height: 30 });
  for (const e of list) {
    if (e.kind === 'file') continue;   // files repeat their messages' links; draw each link once, from the message
    for (const r of e.from || []) if (ids.has(r.ref)) edges.push({ id: `${r.ref}>${e.id}`, source: r.ref, target: e.id });
    for (const r of e.to || []) if (ids.has(r.ref)) edges.push({ id: `${e.id}>${r.ref}`, source: e.id, target: r.ref });
  }
  for (const ed of edges) g.setEdge(ed.source, ed.target);
  L.dagre.layout(g);
  const nodes = list.map((e) => {
    const p = g.node(e.id);
    return { id: e.id, position: { x: p.x - p.width / 2, y: p.y - p.height / 2 }, data: { label: e.title, status: e.status, kind: e.kind }, width: p.width, height: 30 };
  });
  return { nodes, edges };
}

function makeApp(L) {
  const { createElement: h, useEffect: uE, useMemo, ReactFlow, ReactFlowProvider, Background, Controls, MiniMap, useReactFlow, MarkerType } = L;
  function Canvas({ base, sel, onOpen }) {
    const rf = useReactFlow();
    const near = useMemo(() => {
      const s = new Set(sel ? [sel] : []);
      for (const e of base.edges) if (e.source === sel || e.target === sel) { s.add(e.source); s.add(e.target); }
      return s;
    }, [base, sel]);
    const nodes = useMemo(() => base.nodes.map((n) => {
      const on = n.id === sel, lit = near.has(n.id), c = COL[n.data.status];
      return { ...n, selected: on, style: {
        width: n.width, height: 30, padding: '4px 10px', fontSize: 12.5, borderRadius: n.data.kind === 'file' ? 8 : 15, color: '#e8eaee',
        background: on ? 'rgba(255,255,255,.16)' : 'rgba(30,32,38,.92)', border: `1.4px solid ${on ? '#ffffff' : c || (n.data.kind === 'file' ? '#5a6070' : '#3e4450')}`,
        opacity: sel && !lit ? 0.35 : 1, fontWeight: on ? 600 : 400, boxShadow: on ? '0 0 0 4px rgba(255,255,255,.12)' : 'none' } };
    }), [base, sel, near]);
    const edges = useMemo(() => base.edges.map((e) => {
      const lit = sel && (e.source === sel || e.target === sel);
      return { ...e, animated: !!lit, markerEnd: { type: MarkerType.ArrowClosed, width: 14, height: 14, color: lit ? '#ffffff' : '#555b66' },
        style: { stroke: lit ? '#ffffff' : '#555b66', strokeWidth: lit ? 1.8 : 1, opacity: sel && !lit ? 0.25 : 0.9 } };
    }), [base, sel]);
    uE(() => {
      const n = sel && base.nodes.find((x) => x.id === sel);
      if (n) rf.setCenter(n.position.x + n.width / 2, n.position.y + 15, { zoom: Math.max(rf.getZoom(), 0.9), duration: 600 });
    }, [sel]);
    return h(ReactFlow, { nodes, edges, onNodeClick: (_, n) => onOpen(n.id), fitView: !sel, minZoom: 0.05, maxZoom: 2, colorMode: 'dark',
      nodesConnectable: false, proOptions: { hideAttribution: true }, onlyRenderVisibleElements: true },
      h(Background, { gap: 22, size: 1, color: '#3a3f48' }), h(Controls, { showInteractive: false }),
      h(MiniMap, { pannable: true, zoomable: true, nodeColor: (n) => COL[n.data.status] || (n.data.kind === 'file' ? '#6b7280' : '#9aa0a8'), maskColor: 'rgba(0,0,0,.55)' }));
  }
  return (props) => h(ReactFlowProvider, null, h(Canvas, props));
}

registerView('flowmap', (props) => {
  const { c, useBind } = props;
  const nav = useNav(props);
  const { list } = useEntities(props);
  const box = useRef(null), root = useRef(null), lib = useRef(null), base = useRef({ list: null, g: null }), App = useRef(null);
  const openRef = useRef(nav.open); openRef.current = nav.open;

  useEffect(() => {
    let gone = false;
    loadLib().then((L) => {
      if (gone || !box.current) return;
      lib.current = L; App.current = makeApp(L);
      root.current = L.createRoot(box.current);
      draw();
    });
    return () => { gone = true; if (root.current) { root.current.unmount(); root.current = null; } };
  }, []);
  const draw = () => {
    const L = lib.current;
    if (!L || !root.current || !list.length) return;
    if (base.current.list !== list) base.current = { list, g: graphOf(L, list) };
    root.current.render(L.createElement(App.current, { base: base.current.g, sel: nav.sel, onOpen: (id) => openRef.current(id) }));
  };
  useEffect(draw, [list, nav.sel]);

  const e = nav.sel ? list.find((x) => x.id === nav.sel) : null;
  return html`
    <div class="nav-head"><span class="ttl">${useBind(c.title) ?? 'Flow'}</span>
      <span class="end"><span class="meta">${e ? `${(e.from || []).length} in, ${(e.to || []).length} out` : `${list.length} items`}</span></span></div>
    <div class="fm-canvas" data-hit="" data-nodrag="" ref=${box}></div>`;
}, { cls: 'c-flowmap' });
