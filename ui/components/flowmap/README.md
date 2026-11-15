# flowmap

The whole entity graph on React Flow (`ui/vendor/reactflow`: React, ReactDOM, React Flow and dagre in one pinned bundle). Pan, scroll,
zoom, minimap, drag any node. dagre lays it out left to right from each entity's `from` / `to` (each link drawn once, from the message).
Click a node: it opens in every workbench card (`nav`); the open node lights up, its links animate, the rest dims and the view glides to it.
React renders into its own root inside this Preact host.

`{type:"flowmap", entities, nav:"@/nav", title:"Flow"}`; size tokens `fm-w`, `fm-h`.

The canvas/editor carries `data-nodrag` (morc bridge.js): dragging inside it pans, moves nodes or selects text; the card moves only from its edges and title.
