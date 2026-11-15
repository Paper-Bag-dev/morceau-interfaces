# Vendored libraries (served locally; the widgets never touch the network)

| library | version | license | source | sha256 (min build) |
|---|---|---|---|---|
| Cytoscape.js | 3.34.4 | MIT (cytoscape.LICENSE) | npm cytoscape@3.34.4 (github.com/cytoscape/cytoscape.js), tarball integrity checked by 
pm pack, file dist/cytoscape.min.js | 081591AB735B2B6A3B31A3C60F8F3A3E2FEE8C92939E0C7820AD1208571B6CD8 |

To update: 
pm pack cytoscape@<version> in an empty folder, copy dist/cytoscape.min.js + LICENSE here, update this table, check the new version has been public for 2+ weeks.
Loaded lazily by components/graph.js (only when a graph is shown).
| three.js | 0.186.1 | MIT (three/LICENSE) | npm `three@0.186.1` (github.com/mrdoob/three.js), tarball integrity checked by `npm pack`; files `build/three.module.js`, `build/three.core.js`, `examples/jsm/controls/OrbitControls.js`, `examples/jsm/environments/RoomEnvironment.js` | three.module.js `9052042D676CB0FDC1DDFEFE193053F34B7AC0513A616FDAC4535D49987812EA`; three.core.js `9EDDE002B066A9A05676A6127F67735B62BAF399BDEA529F2F7E31657DA769E6` |

three.js is wired through an import map in `index.html` (`three` and `three/addons/`), used by `plugins/model3d.js` and loaded only when a 3D model is shown. 0.186.1 was pinned (published 2026-09-24) rather than the newest 0.187.0 (under two weeks old when vendored). Add another addon the same way: copy the file under `vendor/three/addons/...`.| Preact | 10.24.3 | MIT (preact/LICENSE-preact) | npm `preact@10.24.3`, files `dist/preact.module.js`, `hooks/dist/hooks.module.js` | preact.module.js `9BA8BB591B0C9A72132FF65A76E69C5A4E49CF71721B47B659F3243658BDE05B`; hooks.module.js `896FC8E546B96C3FCA29743B493293820AD4E76396FD36FF05F18A52EAF303E1` |
| htm | 3.1.1 | Apache-2.0 (preact/LICENSE-htm) | npm `htm@3.1.1`, file `dist/htm.module.js` | `AB33DD3F38059B9BE4D5F5350128EEFB2356639C4E0BBE9D9E8B3BA75847E9E4` |
| Tailwind CSS (browser build) | 4.1.11 | MIT (tailwind/LICENSE) | npm `@tailwindcss/browser@4.1.11`, file `dist/index.global.js` renamed `tailwind.browser.js` | `7933E213B39F3434B04EB24D219F279596A9C7D9F7833F7E3A8DB3CEBE7E69D3` |

Preact + htm are wired through the import map in `index.html` (`preact`, `preact/hooks`, `htm`) and used by `core/view.js` (Preact components, docs/views.md). Tailwind's browser build generates CSS for the classes on the page at runtime; `core/view.js` loads it only on a card that uses a `class` (preflight left out, so existing components keep their look). All three pinned to versions long public when vendored (12 Nov 2026).
| Monaco Editor | 0.57.0 | MIT (monaco/LICENSE, monaco/ThirdPartyNotices.txt) | npm `monaco-editor@0.57.0` (published 2026-09-24), `npm pack`, folder `min/vs` copied as `monaco/vs` | loader.js `35B59DF80B41A8B73BF1CEB159B44F040567C2AB53F4E77A3D002D9DC39D55D1`; editor/editor.main.js `60B192EB811710DCC4090FDDF63AC5365B46E538DC73C51B6A27C8EAFFC48B12` |

Monaco (11-14) is the code editor behind the workbench `codeedit` component: syntax highlighting, search, multi-cursor, the VS Code keys. Loaded lazily with its own AMD loader (`monaco/vs/loader.js`, `require(['vs/editor/editor.main'])`) only when an editor card opens; the language workers run from the same origin. 0.57.0 pinned (the newest when vendored, public 7 weeks).
| React Flow (+ React, ReactDOM, dagre) | @xyflow/react 12.12.0, react / react-dom 19.1.10, @dagrejs/dagre 3.1.3 | MIT / ISC (reactflow/LICENSES.txt, every package inside) | `npm install` of those pins, bundled once with esbuild 0.28.2 by `reactflow/build.mjs` from `reactflow/entry.js` | reactflow.bundle.js `7EF5D38C9CAB077019F743C1EF8E27831C9FBECD53F4408ED78D27D74E6FD14F` |

React Flow (11-14) draws the workbench `flowmap` canvas: the whole entity graph, pan, zoom, minimap, draggable nodes; dagre lays it out. React has no ESM build, so React, ReactDOM, React Flow (with zustand, classcat, d3-*) and dagre are one pinned ESM bundle (`reactflow.bundle.js`, its stylesheet `reactflow.css`), loaded only when a flowmap card opens. React runs inside the Preact component's own root; the two never share a tree. To update: same `npm install` with new pins (public 2+ weeks), `node build.mjs` beside a node_modules with the new pins, refresh LICENSES.txt and this row.
