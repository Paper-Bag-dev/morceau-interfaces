// A procedural naked-style motorcycle, built from three.js primitives (no model file, no licence, fully repaintable).
// Contract for any model module: export build(THREE, opts) -> {group, parts:[{name,label,info,group,explode}], setPaint(hex), highlight(name|null, hex)}.
// Axes: the bike points along +X, up is +Y, its centre plane is z = 0; wheels touch y = 0. Units are roughly metres (a 1.4 m wheelbase).

export const meta = { name: 'Naked sport bike', note: 'procedural, built from primitives' };

export function build(T, opts = {}) {
  const root = new T.Group();
  const V = (x, y, z) => new T.Vector3(x, y, z);
  const std = (color, metalness, roughness, extra = {}) => new T.MeshStandardMaterial({ color, metalness, roughness, ...extra });
  const M = {
    steel: std(0x30343b, 0.8, 0.38), chrome: std(0xd9dde3, 1, 0.16), rubber: std(0x131418, 0, 0.9), engine: std(0x3d424b, 0.85, 0.42),
    seat: std(0x1a1a1e, 0, 0.75), spring: std(0xe0b030, 0.5, 0.35),
    glass: std(0xffffff, 0, 0.1, { emissive: new T.Color(0xfff1c9), emissiveIntensity: 0.7 }),
    red: std(0xb3202a, 0.2, 0.4, { emissive: new T.Color(0x990011), emissiveIntensity: 0.5 }),
    paint: opts.lite ? std(opts.paint || '#f4764e', 0.35, 0.34) : new T.MeshPhysicalMaterial({ color: opts.paint || '#f4764e', metalness: 0.35, roughness: 0.32, clearcoat: 1, clearcoatRoughness: 0.12 }),
  };
  const mesh = (geo, mat) => { const m = new T.Mesh(geo, mat); m.castShadow = true; return m; };
  const tube = (a, b, r, mat, seg = 12) => {
    const A = V(...a), B = V(...b), d = B.clone().sub(A), m = mesh(new T.CylinderGeometry(r, r, d.length(), seg), mat);
    m.position.copy(A).add(B).multiplyScalar(0.5);
    m.quaternion.setFromUnitVectors(V(0, 1, 0), d.normalize());
    return m;
  };
  const blob = (sx, sy, sz, mat, x, y, z) => { const m = mesh(new T.SphereGeometry(1, 28, 18), mat); m.scale.set(sx, sy, sz); m.position.set(x, y, z); return m; };
  const box = (w, h, d, mat, x, y, z) => { const m = mesh(new T.BoxGeometry(w, h, d), mat); m.position.set(x, y, z); return m; };
  const cyl = (r, len, mat, x, y, z) => { const m = mesh(new T.CylinderGeometry(r, r, len, 28), mat); m.rotation.x = Math.PI / 2; m.position.set(x, y, z); return m; };   // axis along Z
  const both = (fn) => [fn(1), fn(-1)].flat();                                                                                          // mirror across the centre plane

  const parts = [];
  const part = (name, label, info, explode, ...objs) => {
    const g = new T.Group();
    g.name = name;
    objs.flat().forEach((o) => g.add(o));
    root.add(g);
    parts.push({ name, label, info, group: g, explode, base: g.position.clone() });
  };

  // wheels: torus tyre + rim + ten spokes + hub + brake disc; the axle runs along Z
  const wheel = (x) => {
    const g = new T.Group();
    const tire = mesh(new T.TorusGeometry(0.30, 0.078, 22, 64), M.rubber);
    const rim = mesh(new T.TorusGeometry(0.235, 0.016, 12, 64), M.chrome);
    const hub = cyl(0.05, 0.13, M.chrome, 0, 0, 0);
    const disc = cyl(0.17, 0.012, M.chrome, 0, 0, 0.075);
    g.add(tire, rim, hub, disc);
    for (let i = 0; i < 10; i++) {
      const a = (i / 10) * Math.PI * 2, s = mesh(new T.CylinderGeometry(0.008, 0.008, 0.19, 6), M.chrome);
      s.position.set(Math.cos(a) * 0.14, Math.sin(a) * 0.14, 0);
      s.rotation.z = a - Math.PI / 2;
      g.add(s);
    }
    g.position.set(x, 0.378, 0);
    return g;
  };
  part('wheelF', 'Front wheel', '17 inch, 120/70 tyre, single disc visible side', [0.75, 0, 0], wheel(0.66));
  part('wheelR', 'Rear wheel', '17 inch, 180/55 tyre, drives the chain', [-0.75, 0, 0], wheel(-0.62));

  part('forks', 'Front forks', 'Upright telescopic forks, chrome stanchions', [0.4, 0.28, 0],
    both((s) => [tube([0.30, 0.97, 0.085 * s], [0.50, 0.64, 0.085 * s], 0.021, M.chrome), tube([0.50, 0.64, 0.085 * s], [0.66, 0.378, 0.085 * s], 0.03, M.steel)]),
    box(0.07, 0.04, 0.22, M.steel, 0.29, 0.97, 0));

  part('handlebar', 'Handlebar', 'Wide single bar with mirrors', [0.15, 0.62, 0],
    both((s) => [tube([0.27, 1.03, 0.05 * s], [0.20, 1.075, 0.30 * s], 0.011, M.steel), cyl(0.016, 0.1, M.rubber, 0.20, 1.075, 0.33 * s),
      tube([0.22, 1.07, 0.26 * s], [0.12, 1.19, 0.28 * s], 0.005, M.steel), blob(0.026, 0.02, 0.04, M.steel, 0.12, 1.2, 0.28 * s)]).flat());

  part('light', 'Headlight', 'Round LED headlight', [0.6, 0.18, 0],
    blob(0.075, 0.095, 0.095, M.glass, 0.40, 0.93, 0), tube([0.30, 0.92, 0], [0.37, 0.93, 0], 0.012, M.steel));

  const rails = [[[0.28, 0.90], [-0.10, 0.84]], [[-0.10, 0.84], [-0.45, 0.81]], [[0.27, 0.88], [0.05, 0.40]], [[0.05, 0.40], [-0.22, 0.42]], [[-0.10, 0.84], [-0.22, 0.42]], [[0.18, 0.66], [-0.04, 0.62]], [[-0.45, 0.81], [-0.64, 0.60]]];
  part('frame', 'Trellis frame', 'Steel tubes, steering head to swingarm pivot', [0, 0.12, 0],
    tube([0.29, 0.90, 0], [0.31, 1.0, 0], 0.035, M.steel), tube([0.27, 0.88, -0.09], [0.27, 0.88, 0.09], 0.016, M.steel), tube([-0.10, 0.84, -0.10], [-0.10, 0.84, 0.10], 0.014, M.steel),
    ...rails.map(([a, b]) => both((s) => tube([a[0], a[1], 0.10 * s], [b[0], b[1], 0.10 * s], 0.015, M.steel))));

  part('swingarm', 'Swingarm and shock', 'Rear shock with a gold spring', [-0.4, -0.12, 0],
    both((s) => tube([-0.24, 0.42, 0.12 * s], [-0.62, 0.378, 0.115 * s], 0.03, M.steel)), tube([-0.42, 0.40, -0.12], [-0.42, 0.40, 0.12], 0.02, M.steel),
    tube([-0.20, 0.74, 0], [-0.30, 0.46, 0], 0.036, M.spring), tube([-0.30, 0.46, 0], [-0.33, 0.40, 0], 0.024, M.steel));

  part('engine', 'Engine', 'Inline-twin, liquid cooled, under the tank', [0, -0.38, 0],
    box(0.30, 0.24, 0.26, M.engine, 0.02, 0.36, 0), (() => { const h = box(0.22, 0.26, 0.2, M.engine, 0.10, 0.58, 0); h.rotation.z = -0.35; return h; })(),
    box(0.12, 0.05, 0.19, M.steel, 0.12, 0.72, 0), cyl(0.12, 0.34, M.engine, -0.05, 0.30, 0), cyl(0.09, 0.30, M.engine, -0.17, 0.33, 0),
    cyl(0.1, 0.03, M.chrome, -0.03, 0.32, 0.18), box(0.04, 0.26, 0.32, M.steel, 0.31, 0.55, 0));

  const header = new T.CatmullRomCurve3([V(0.20, 0.50, 0.09), V(0.30, 0.34, 0.14), V(0.10, 0.22, 0.17), V(-0.20, 0.22, 0.19), V(-0.48, 0.34, 0.20)]);
  part('exhaust', 'Exhaust', 'Stainless header into a short muffler', [-0.12, -0.2, 0.5],
    mesh(new T.TubeGeometry(header, 64, 0.022, 12, false), M.chrome), tube([-0.30, 0.28, 0.20], [-0.72, 0.43, 0.19], 0.058, M.chrome), tube([-0.72, 0.43, 0.19], [-0.74, 0.435, 0.19], 0.05, M.steel));

  part('tank', 'Fuel tank', '14 litres, clear-coated paint (change it with the swatches)', [0, 0.55, 0],
    blob(0.30, 0.15, 0.17, M.paint, 0.07, 0.97, 0), blob(0.17, 0.12, 0.15, M.paint, 0.22, 0.95, 0));

  part('seat', 'Seat and tail', 'Two-up seat with a short painted tail', [-0.25, 0.5, 0],
    blob(0.30, 0.045, 0.14, M.seat, -0.28, 0.885, 0), (() => { const t = blob(0.26, 0.075, 0.13, M.paint, -0.58, 0.905, 0); t.rotation.z = 0.18; return t; })(), box(0.02, 0.03, 0.09, M.red, -0.82, 0.92, 0));

  const arc = (x, rot, mat) => { const m = mesh(new T.TorusGeometry(0.345, 0.012, 8, 32, Math.PI * 0.62), mat); m.scale.z = 6; m.position.set(x, 0.378, 0); m.rotation.z = rot; return m; };
  part('fenders', 'Fenders', 'Short front fender and a rear hugger', [0, 0.34, 0], arc(0.66, Math.PI * 0.2, M.paint), arc(-0.62, Math.PI * 0.18, M.steel));

  // every mesh gets its own material (so one part can glow on hover without lighting its neighbours) and remembers its part
  const meshes = [];
  for (const p of parts) p.group.traverse((o) => {
    if (!o.isMesh) return;
    const isPaint = o.material === M.paint;
    o.material = o.material.clone();
    o.userData = { part: p.name, paint: isPaint, ei: o.material.emissiveIntensity ?? 0, em: o.material.emissive ? o.material.emissive.clone() : null };
    meshes.push(o);
  });

  return {
    group: root,
    parts,
    meshes,
    setPaint(hex) { for (const m of meshes) if (m.userData.paint) m.material.color.set(hex); },
    highlight(name, hex) {
      for (const m of meshes) {
        if (!m.material.emissive) continue;
        if (name && m.userData.part === name) { m.material.emissive.set(hex); m.material.emissiveIntensity = 0.55; }
        else { m.material.emissive.copy(m.userData.em); m.material.emissiveIntensity = m.userData.ei; }
      }
    },
  };
}
