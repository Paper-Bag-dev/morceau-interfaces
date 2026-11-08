# Vendored libraries (served locally; the widgets never touch the network)

| library | version | license | source | sha256 (min build) |
|---|---|---|---|---|
| Cytoscape.js | 3.34.4 | MIT (cytoscape.LICENSE) | npm cytoscape@3.34.4 (github.com/cytoscape/cytoscape.js), tarball integrity checked by 
pm pack, file dist/cytoscape.min.js | 081591AB735B2B6A3B31A3C60F8F3A3E2FEE8C92939E0C7820AD1208571B6CD8 |

To update: 
pm pack cytoscape@<version> in an empty folder, copy dist/cytoscape.min.js + LICENSE here, update this table, check the new version has been public for 2+ weeks.
Loaded lazily by components/graph.js (only when a graph is shown).
| three.js | 0.186.1 | MIT (three/LICENSE) | npm `three@0.186.1` (github.com/mrdoob/three.js), tarball integrity checked by `npm pack`; files `build/three.module.js`, `build/three.core.js`, `examples/jsm/controls/OrbitControls.js`, `examples/jsm/environments/RoomEnvironment.js` | three.module.js `9052042D676CB0FDC1DDFEFE193053F34B7AC0513A616FDAC4535D49987812EA`; three.core.js `9EDDE002B066A9A05676A6127F67735B62BAF399BDEA529F2F7E31657DA769E6` |

three.js is wired through an import map in `index.html` (`three` and `three/addons/`), used by `plugins/model3d.js` and loaded only when a 3D model is shown. 0.186.1 was pinned (published 2026-09-24) rather than the newest 0.187.0 (under two weeks old when vendored). Add another addon the same way: copy the file under `vendor/three/addons/...`.