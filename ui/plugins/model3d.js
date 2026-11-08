import { register } from '../core/registry.js';
import { reduced } from '../core/motion.js';
import { accent } from '../charts/util.js';
import { working } from '../core/working.js';

// {type:"model3d", model:"bike", width:560, height:340, spin:{bind:"/spin"}, explode:{bind:"/explode"}, paint:{bind:"/paint"}, hover:"/part", pick:"/picked"}
// A plugin: three.js is vendored locally and loaded only when a model is shown (import map in index.html). The scene is a transparent
// canvas (just the model and its shadow), lit by a studio environment. Orbit by dragging, wheel zooms, hover lights a part, a click
// picks it. Rendering is on demand: the loop only runs while something moves (orbit damping, explode tween, spin), so an idle model costs nothing.
// Model modules live in ui/models/<name>.js (contract in models/bike.js).

let libP = null;
const lib = () => (libP ||= Promise.all([import('three'), import('three/addons/controls/OrbitControls.js'), import('three/addons/environments/RoomEnvironment.js')])
  .then(([T, oc, re]) => ({ T, OrbitControls: oc.OrbitControls, RoomEnvironment: re.RoomEnvironment })));

const hex = (h, s, l) => {
  s /= 100; l /= 100;
  const k = (n) => (n + h / 30) % 12, a = s * Math.min(l, 1 - l), f = (n) => Math.round(255 * (l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)))));
  return (f(0) << 16) | (f(8) << 8) | f(4);
};

register('model3d', (c, ctx) => {
  const lite = c.quality === 'lite';   // no shadow map / environment map / clear-coat: the safe mode if a GPU or driver drops the context
  const W = c.width || 560, H = c.height || 340, HINT = 'Drag to turn it, scroll to zoom, click a part.';
  const el = document.createElement('div');
  el.className = 'c-model';
  el.dataset.hit = '';
  const frame = document.createElement('div');
  frame.className = 'mframe';
  frame.style.width = W + 'px';
  frame.style.height = H + 'px';
  const inner = document.createElement('div');                 // counter-zoomed: three.js sees ordinary 1:1 pointer coordinates
  inner.className = 'mcanvas';
  frame.append(inner);
  const info = document.createElement('div');
  info.className = 'ginfo plate';
  info.textContent = HINT;
  el.append(frame, info);
  if (c.info === false) info.style.display = 'none';           // a pet needs no read-out

  const pulse = working(el);                                   // blue pulse while three.js loads and the first frame renders
  const fit = () => { const z = window.__zoom || 1; inner.style.zoom = String(1 / z); inner.style.width = W * z + 'px'; inner.style.height = H * z + 'px'; };
  fit();

  const st = { spin: false, target: 0, now: 0, paint: null };
  let T, renderer, scene, camera, controls, built, raycaster, frames = 0, raf = 0, inFrame = false, lost = false, saved = null, recoveries = 0, last = 0, lastDraw = 0, introFrom = null, introTo = null, intro = 0, started = false, hoverName = '', down = null;
  const pointer = { x: 0, y: 0 };

  ctx.bind(c.spin, (v) => { st.spin = !!v; wake(); });
  ctx.bind(c.explode, (v) => { st.target = v === true ? 1 : typeof v === 'number' ? v : 0; wake(); });
  ctx.bind(c.paint, (v) => { st.paint = v; if (built && v) { built.setPaint(v); wake(); } });

  const applyExplode = () => { for (const p of built.parts) p.group.position.set(p.base.x + p.explode[0] * st.now, p.base.y + p.explode[1] * st.now, p.base.z + p.explode[2] * st.now); };
  const partAt = (e) => {
    const r = renderer.domElement.getBoundingClientRect();
    pointer.x = ((e.clientX - r.left) / r.width) * 2 - 1;
    pointer.y = -((e.clientY - r.top) / r.height) * 2 + 1;
    raycaster.setFromCamera(pointer, camera);
    const hit = raycaster.intersectObject(built.group, true)[0];
    return hit ? hit.object.userData.part : '';
  };
  const accentHex = () => { const a = accent(); return hex(a.h, a.s, a.l); };

  function frameLoop(now) {
    raf = 0;
    inFrame = true;                       // OrbitControls fires 'change' inside update(): that must not start a second loop
    try { step(now); } finally { inFrame = false; }
  }
  function step(now) {
    if (!renderer || !scene || !camera || !controls) return;       // torn down (or mid-rebuild): nothing to draw
    if (!el.isConnected) { ctx.send({ t: 'log', msg: 'model3d: element disconnected, disposing (our own forceContextLoss)' }); return dispose(); }
    // Animated models (a pet) run continuously but are capped (default 30 fps) so an idle character stays cheap.
    if (built.animated && now - lastDraw < 1000 / (c.fps || 30) - 2) { raf = requestAnimationFrame(frameLoop); return; }
    const dt = Math.max(0, Math.min(0.05, (now - last) / 1000));   // never negative (wake() stamps last with a later clock reading than the frame time)
    last = now;
    lastDraw = now;
    let busy = false;
    if (built.animated) { busy = true; if (built.update) built.update(dt, now / 1000); }
    if (intro < 1) {                                            // arrive: the camera swings in from a wide, high angle
      intro = Math.max(0, Math.min(1, intro + dt / 1.1));
      const e = 1 - Math.pow(1 - intro, 3);
      camera.position.lerpVectors(introFrom, introTo, reduced ? 1 : e);
      busy = true;
    }
    if (Math.abs(st.target - st.now) > 0.002) { st.now += (st.target - st.now) * (1 - Math.exp(-7 * dt)); busy = true; applyExplode(); }
    else if (st.now !== st.target) { st.now = st.target; applyExplode(); }
    controls.autoRotate = st.spin && !reduced;
    const ctlMoved = controls.update(dt);
    if (ctlMoved) busy = true;
    if (controls.autoRotate) busy = true;
    const t0 = performance.now();
    renderer.render(scene, camera);
    pulse.done();                                               // first frame is up: one ring, then it stops
    if (window.__debug && frames++ < 6) ctx.send({ t: 'log', msg: 'model3d frame ' + frames + ': ' + (performance.now() - t0).toFixed(0) + ' ms (calls ' + renderer.info.render.calls + ', tris ' + renderer.info.render.triangles + ')' });
    if (busy) raf = requestAnimationFrame(frameLoop);
    if (window.__debug) { fps.n++; fps.why = { intro: +intro.toFixed(2), explode: +(st.target - st.now).toFixed(3), spin: st.spin, ctl: ctlMoved, busy }; }
  }
  function wake() { if (!raf && !inFrame && renderer && controls && built) { last = performance.now(); raf = requestAnimationFrame(frameLoop); } }

  const fps = { n: 0, why: null };
  if (window.__debug) setInterval(() => { if (el.isConnected) { ctx.send({ t: 'log', msg: 'model3d fps ' + (fps.n / 3).toFixed(1) + ' why ' + JSON.stringify(fps.why) }); fps.n = 0; } }, 3000);
  /** The GPU process reset under us (seen on some Intel setups): rebuild renderer + scene from scratch, then put the camera back. */
  function recover() {
    if (recoveries++ > 8 || !el.isConnected) return;
    saved = { pos: camera.position.clone(), target: controls.target.clone() };
    ctx.send({ t: 'log', msg: 'model3d: rebuilding after context loss (#' + recoveries + ')' });
    dispose(true);
    started = false; lost = false; intro = 1;
    init().then(() => { if (saved && camera) { camera.position.copy(saved.pos); controls.target.copy(saved.target); controls.update(); wake(); } });
  }

  function dispose(keepElement) {
    if (!renderer) return;
    ctx.send({ t: 'log', msg: 'model3d: dispose() called' });
    cancelAnimationFrame(raf);
    scene.traverse((o) => { if (o.geometry) o.geometry.dispose(); const ms = o.material ? [o.material].flat() : []; for (const m of ms) { for (const v of Object.values(m)) if (v && v.isTexture) v.dispose(); m.dispose(); } });
    controls.dispose(); renderer.dispose();
    if (!keepElement) renderer.forceContextLoss(); else renderer.domElement.remove();
    renderer = null;
  }

  async function init() {
    if (started) return;
    started = true;
    try {
      const L = await lib();
      T = L.T;
      const mod = await import('../models/' + c.model + '.js');
      const z = window.__zoom || 1;
      renderer = new T.WebGLRenderer({ antialias: false, alpha: true })   // MSAA is the costly part under software GL   // default power preference: 'high-performance' can trigger GPU switching on dual-GPU laptops;
      renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
      renderer.setSize(W * z, H * z);
      renderer.setClearColor(0x000000, 0);                     // transparent: only the model and its shadow are drawn
      renderer.toneMapping = T.ACESFilmicToneMapping;
      renderer.toneMappingExposure = 1.05;
      renderer.shadowMap.enabled = !lite;
      inner.append(renderer.domElement);
      renderer.domElement.addEventListener('webglcontextlost', (e) => {
        e.preventDefault();                                   // allow the browser to restore it
        ctx.send({ t: 'log', msg: 'model3d: WebGL context LOST' });
        lost = true;
        setTimeout(() => { if (lost) recover(); }, 900);        // not restored in time: rebuild everything ourselves
      });
      renderer.domElement.addEventListener('webglcontextrestored', () => { lost = false; ctx.send({ t: 'log', msg: 'model3d: WebGL context restored' }); wake(); });
      if (window.__debug) {   // MORC_DEBUG: which GPU are we really on, and what does a frame cost?
        const gl = renderer.getContext(), ext = gl.getExtension('WEBGL_debug_renderer_info');
        ctx.send({ t: 'log', msg: 'model3d GPU: ' + (ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : 'unknown') + ' | canvas ' + renderer.domElement.width + 'x' + renderer.domElement.height });
      }

      scene = new T.Scene();
      if (lite) { scene.add(new T.HemisphereLight(0xffffff, 0x556070, 1.6), new T.AmbientLight(0xffffff, 0.5)); }
      else {
        const pm = new T.PMREMGenerator(renderer);
        scene.environment = pm.fromScene(new L.RoomEnvironment(), 0.04).texture;
        scene.environmentIntensity = 0.95;
        pm.dispose();
      }
      camera = new T.PerspectiveCamera(32, W / H, 0.1, 50);
      camera.position.set(3.8, 2.0, 3.5);
      const key = new T.DirectionalLight(0xffffff, 2.2);
      key.position.set(2.5, 4, 2);
      key.castShadow = !lite;
      key.shadow.mapSize.set(512, 512);
      Object.assign(key.shadow.camera, { left: -2, right: 2, top: 2, bottom: -2, near: 0.5, far: 10 });
      key.shadow.bias = -0.0005;
      const rimLight = new T.DirectionalLight(accentHex(), 1.4);   // a coral edge light, so the silhouette separates from any page behind
      rimLight.position.set(-3, 2, -2);
      scene.add(key, rimLight);
      if (!lite) {
        const ground = new T.Mesh(new T.PlaneGeometry(8, 8), new T.ShadowMaterial({ opacity: 0.3 }));
        ground.rotation.x = -Math.PI / 2;
        ground.receiveShadow = true;
        scene.add(ground);
      }

      built = mod.build(T, { paint: st.paint || '#f4764e', lite, props: c });   // props: the component's own fields (a glTF model reads src from it)
      built.onLoad = () => wake();                                                // an async model (glTF) asks for a repaint when it arrives
      if (st.paint) built.setPaint(st.paint);
      scene.add(built.group);
      applyExplode();

      // a model may say how it wants to be framed: view = {position:[x,y,z], target:[x,y,z]}
      const view = built.view || {};
      introTo = new T.Vector3(...(view.position || [2.45, 1.2, 2.25]));
      introFrom = introTo.clone().multiplyScalar(1.55).add(new T.Vector3(0, 0.4, 0));
      camera.position.copy(introFrom);
      controls = new L.OrbitControls(camera, renderer.domElement);
      Object.assign(controls, { enableDamping: true, dampingFactor: 0.08, enablePan: false, enableZoom: c.zoom !== false, minDistance: 1.3, maxDistance: 6, maxPolarAngle: Math.PI / 2 - 0.04, autoRotateSpeed: 2.2 });
      controls.target.set(...(view.target || [0, 0.5, 0]));
      controls.addEventListener('change', wake);
      raycaster = new T.Raycaster();

      const cv = renderer.domElement;
      cv.addEventListener('pointermove', (e) => {
        if (e.buttons) return;                                   // orbiting: no hover work
        const n = partAt(e);
        if (n !== hoverName) {
          hoverName = n;
          built.highlight(n, accentHex());
          const p = built.parts.find((x) => x.name === n);
          info.textContent = p ? `${p.label}: ${p.info}` : HINT;
          if (c.hover) ctx.store.set(c.hover, n);
          cv.style.cursor = n ? 'pointer' : 'grab';
          wake();
        }
      });
      cv.addEventListener('pointerleave', () => { if (hoverName) { hoverName = ''; built.highlight(null); info.textContent = HINT; wake(); } });
      cv.addEventListener('pointerdown', (e) => { down = { x: e.clientX, y: e.clientY }; });
      cv.addEventListener('pointerup', (e) => {                  // a click is a press that barely moved (otherwise it was an orbit)
        if (down && Math.hypot(e.clientX - down.x, e.clientY - down.y) < 4) {
          const n = partAt(e);
          if (n && c.pick) ctx.store.set(c.pick, n);
        }
        down = null;
      });
      if (!saved) cv.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 450, easing: 'ease-out' });   // no fade on a rebuild
      wake();
    } catch (e) { info.textContent = String(e.message || e); ctx.send({ t: 'log', msg: 'model3d: ' + (e.stack || e.message || e) }); }
  }

  if (document.getElementById('stage').dataset.entered) init(); else addEventListener('morc-enter', init, { once: true });
  addEventListener('morc-zoom', function onZoom() {
    if (!el.isConnected) return removeEventListener('morc-zoom', onZoom);
    fit();
    if (renderer) { const z = window.__zoom || 1; renderer.setSize(W * z, H * z); wake(); }
  });
  return el;
});
