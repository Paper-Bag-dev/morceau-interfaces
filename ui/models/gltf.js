// A glTF model from a file on disk (for example downloaded from Sketchfab with `morc sketchfab get <uid>`).
// Component props arrive as opts.props: src = URL of the .gltf relative to the page, e.g. "models/assets/<uid>/scene.gltf".
// It loads asynchronously: the group is empty at first and the model drops in when ready (onLoad asks model3d for a repaint).
// It is centred, scaled so its longest side is 1.8 units, and stands on the ground. If the file has animations, the first one loops.
// Contract: see bike.js. No parts (nothing to hover or explode), setPaint/highlight do nothing.

export function build(T, opts = {}) {
  const props = opts.props || {};
  const group = new T.Group();
  let mixer = null;
  const rigNodes = [], tmpQ = new T.Quaternion();
  // props.rig = [{match: "regex on the bone name", axis: "x|y|z", amp: radians, freq: Hz, phase: radians added per matched bone}]: a gentle
  // procedural idle for models with a skeleton but no animation clips. Phase per bone makes chains (hair, feathers) ripple like a wave.
  const rules = (Array.isArray(props.rig) ? props.rig : []).map((r) => ({
    re: new RegExp(r.match, 'i'), ax: new T.Vector3(r.axis === 'x' ? 1 : 0, r.axis === 'y' ? 1 : 0, r.axis === 'z' ? 1 : 0),
    amp: r.amp || 0.05, w: 2 * Math.PI * (r.freq || 0.3), ph: r.phase || 0, n: 0,
  }));
  const api = {
    group,
    parts: [],
    meshes: [],
    animated: false,
    onLoad: null,
    update(dt, t) {
      if (mixer) mixer.update(dt);
      for (const n of rigNodes) {                    // procedural idle for rigged models that ship no clips (see props.rig)
        tmpQ.setFromAxisAngle(n.r.ax, n.r.amp * Math.sin(n.r.w * t + n.r.ph * n.i));
        n.o.quaternion.copy(n.q0).multiply(tmpQ);
      }
    },
    view: props.view && props.view.position ? props.view : { position: [2.7, 1.3, 2.7], target: [0, 0.55, 0] },   // props.view = {position:[x,y,z], target:[x,y,z]} zooms/frames
    setPaint() {},
    highlight() {},
  };
  if (!props.src) { console.error('gltf model: no src'); return api; }
  import('three/addons/loaders/GLTFLoader.js')
    .then(({ GLTFLoader }) => new Promise((res, rej) => new GLTFLoader().load(props.src, res, undefined, rej)))
    .then((gltf) => {
      const root = gltf.scene;
      const box = new T.Box3().setFromObject(root), size = box.getSize(new T.Vector3()), center = box.getCenter(new T.Vector3());
      // longest side 1.8, but never taller than 1.45 so tall models (a character) stay inside the camera frame
      const k = Math.min(1.8 / Math.max(size.x, size.y, size.z, 1e-6), 1.45 / Math.max(size.y, 1e-6));
      root.scale.setScalar(k);
      root.position.set(-center.x * k, -box.min.y * k, -center.z * k);
      root.traverse((o) => { if (o.isMesh) { o.castShadow = !opts.lite; api.meshes.push(o); } });
      group.add(root);
      if (rules.length) {
        root.traverse((o) => { if (o.name) for (const r of rules) if (r.re.test(o.name)) rigNodes.push({ o, q0: o.quaternion.clone(), r, i: r.n++ }); });
        if (rigNodes.length) api.animated = true;
      }
      const sample = []; root.traverse((o) => { if (o.name && sample.length < 3 && /spine|wing|hair/i.test(o.name)) sample.push(o.name); });
      if (window.ipc) window.ipc.postMessage(JSON.stringify({ t: 'log', msg: `gltf loaded: ${rules.length} rig rules matched ${rigNodes.length} bones, ${(gltf.animations || []).length} clips; sample names ${sample.join(', ')}` }));
      if (gltf.animations && gltf.animations.length) {
        mixer = new T.AnimationMixer(root);
        const clips = gltf.animations, find = (n) => clips.find((c) => c.name.toLowerCase().includes(String(n).toLowerCase()));
        const want = props.anim;                      // a clip name (substring), "cycle" (idle, with an occasional other clip), or omitted (the first clip)
        if (want === 'cycle') {
          const idle = find('idle') || clips[0], others = clips.filter((c) => c !== idle);
          let cur = mixer.clipAction(idle).play();
          const next = () => setTimeout(() => {
            if (!others.length) return;
            const o = mixer.clipAction(others[Math.floor(Math.random() * others.length)]);
            o.reset().setLoop(T.LoopOnce, 1);
            o.clampWhenFinished = true;
            cur.crossFadeTo(o.play(), 0.3, false);
            const back = (e) => {
              if (e.action !== o) return;
              mixer.removeEventListener('finished', back);
              const i = mixer.clipAction(idle);
              i.reset().play();
              o.crossFadeTo(i, 0.3, false);
              cur = i;
              next();
            };
            mixer.addEventListener('finished', back);
          }, 7000 + Math.random() * 5000);
          next();
        } else {
          mixer.clipAction((want && find(want)) || clips[0]).play();
        }
        api.animated = true;
      }
      if (api.onLoad) api.onLoad();
    })
    .catch((e) => console.error('gltf model failed:', e && e.message ? e.message : e));
  return api;
}
