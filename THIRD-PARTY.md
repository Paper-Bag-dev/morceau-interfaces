# Third-party software

Morceau bundles or builds on the following. Each keeps its own licence.

## Vendored into `ui/vendor/` (served offline, no network use at runtime)
| library | version | licence | notes |
|---|---|---|---|
| three.js (`three.module.js`, `three.core.js`) | 0.186.1 | MIT | `ui/vendor/three/LICENSE` |
| three.js addons: OrbitControls, RoomEnvironment, GLTFLoader, BufferGeometryUtils, SkeletonUtils | 0.186.1 | MIT | copied unmodified from `three/examples/jsm` |
| Cytoscape.js (`cytoscape.min.js`) | 3.34.4 | MIT | `ui/vendor/cytoscape.LICENSE` |

SHA-256 hashes of every vendored file are in `ui/vendor/README.md`; compare them to the upstream npm packages to verify nothing was altered.

## Rust crates (see `Cargo.lock` for exact versions)
`tao` (Apache-2.0), `wry` (Apache-2.0 / MIT), `windows` (MIT / Apache-2.0), `serde` and `serde_json` (MIT / Apache-2.0), and their dependencies.
Run `cargo install cargo-license && cargo license` for the full list.

## Runtime requirement, not bundled
Microsoft Edge WebView2 Runtime (preinstalled on Windows 11 and current Windows 10), under Microsoft's licence.

## Models
No 3D models ship with this repository. `morc sketchfab get <uid>` downloads a model you choose into `ui/models/assets/` (git-ignored); each model keeps the
licence and attribution of its author (usually a Creative Commons licence). Respect it, and keep the credit if you publish or share the result.
