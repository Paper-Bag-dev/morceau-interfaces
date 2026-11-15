import { registerView, html, useEffect, useRef, useState } from '../../core/view.js';
import { useNav, useEntities } from '../nav/nav.js';

// {type:"codeedit", entities:{bind:"/entities"}, nav:"@/nav", open:"wb-open", save:"wb-save"}
// The whole file of the open entity (one with a `file` path) in Monaco (ui/vendor/monaco: highlighting, search, multi-cursor, VS Code keys);
// Ctrl+S saves. No engine path of its own: the card sets /want, then runs the allowlisted `open` action (fixed argv in actions.json), which reads
// /want from the card's persisted state (opts.persist) and answers {file, text} into /doc. Edits go to /draft (persisted the same way); Save runs
// the `save` action, which reads /draft from the state file, backs the file up, writes it and answers {ok, msg} into /saved.
const LANG = { js: 'javascript', mjs: 'javascript', ts: 'typescript', kt: 'kotlin', kts: 'kotlin', py: 'python', json: 'json', md: 'markdown',
  css: 'css', html: 'html', toml: 'ini', rs: 'rust', sh: 'shell', ps1: 'powershell', yml: 'yaml', yaml: 'yaml' };
const langOf = (f) => LANG[(f || '').split('.').pop().toLowerCase()] || 'plaintext';

let monacoP = null;
/** Monaco's own AMD loader, loaded once per page and only when an editor opens. */
function loadMonaco() {
  if (window.monaco) return Promise.resolve(window.monaco);
  monacoP ||= new Promise((ok, fail) => {
    const css = document.createElement('link');
    css.rel = 'stylesheet'; css.href = 'vendor/monaco/vs/editor/editor.main.css';
    document.head.append(css);
    const s = document.createElement('script');
    s.src = 'vendor/monaco/vs/loader.js';
    s.onload = () => { window.require.config({ paths: { vs: 'vendor/monaco/vs' } }); window.require(['vs/editor/editor.main'], () => ok(window.monaco), fail); };
    s.onerror = () => fail(new Error('could not load vendor/monaco'));
    document.head.append(s);
  });
  return monacoP;
}

registerView('codeedit', (props) => {
  const { c, ctx, useBind, act } = props;
  const nav = useNav(props);
  const { byId } = useEntities(props);
  const e = nav.sel ? byId.get(nav.sel) : null;
  const doc = useBind({ bind: '/doc' });
  const saved = useBind({ bind: '/saved' });
  const [busy, setBusy] = useState('');
  const [dirty, setDirty] = useState(false);
  const box = useRef(null), ed = useRef(null), loading = useRef(false);
  const file = e?.file || null;
  const loaded = !!doc && doc.file === file && typeof doc.text === 'string';

  const save = () => {
    if (!ed.current || !dirty) return;
    setBusy('Saving');
    act([{ do: 'set', path: '/saved', value: null }, { do: 'run', id: c.save || 'wb-save', into: '/saved', fail: { '/saved': { ok: false, msg: 'Save failed. Nothing was written.' } } }]);
  };
  const saveRef = useRef(save); saveRef.current = save;

  useEffect(() => {                                   // a file was opened elsewhere (explorer, a chip, a step): load it
    if (!file || (doc && doc.file === file)) return;
    setBusy('Opening');
    act([{ do: 'set', path: '/want', value: { file, at: Date.now() } }, { do: 'run', id: c.open || 'wb-open', into: '/doc', fail: { '/doc': { file, error: 'Could not open it.' } } }]);
  }, [file]);

  useEffect(() => {                                   // the file arrived: show it in Monaco (made once, then only its model changes)
    if (!loaded || !box.current || loading.current) return;
    loading.current = true;
    loadMonaco().then((monaco) => {
      loading.current = false;
      if (!ed.current) {
        ed.current = monaco.editor.create(box.current, { theme: 'vs-dark', automaticLayout: true, minimap: { enabled: false }, fontSize: 13,
          fontFamily: "'Cascadia Mono', Consolas, monospace", scrollBeyondLastLine: false, tabSize: 2, renderWhitespace: 'selection' });
        ed.current.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => saveRef.current());
        ed.current.onDidChangeModelContent(() => {
          const v = ed.current.getValue();
          ctx.store.set('/draft', { file: ed.current.__file, text: v, at: Date.now() });
          setDirty(v !== ed.current.__orig);
        });
      }
      const old = ed.current.getModel();
      ed.current.__file = doc.file; ed.current.__orig = doc.text;
      ed.current.setModel(monaco.editor.createModel(doc.text, langOf(doc.file)));
      if (old) old.dispose();
      setDirty(false); setBusy('');
    }, () => { loading.current = false; setBusy('The editor could not load.'); });
  }, [doc]);

  useEffect(() => {
    if (!saved) return;
    if (saved.ok && ed.current) { ed.current.__orig = ed.current.getValue(); setDirty(false); }
    setBusy('');
  }, [saved]);
  useEffect(() => () => { if (ed.current) { ed.current.getModel()?.dispose(); ed.current.dispose(); ed.current = null; } }, []);

  const msg = !e ? 'Open a file to edit it here.' : !file ? `${e.title} is not a file. Open one of the files it links to.`
    : doc?.error && doc.file === file ? doc.error : '';
  return html`
    <div class="ce-bar">
      <span class="path" title=${file || ''}>${e ? e.path || e.title : ''}</span>
      ${dirty && html`<span class="dot" title="Unsaved changes"></span>`}
      <span class="msg">${msg || busy || (saved && !dirty ? saved.msg : '')}</span>
      <button class="ce-btn" data-hit="" disabled=${!dirty || !!busy} title="Undo every change since it was opened" onClick=${() => ed.current && ed.current.setValue(ed.current.__orig)}>Revert</button>
      <button class="ce-btn primary" data-hit="" disabled=${!dirty || !!busy} title="Save (Ctrl+S)" onClick=${save}>Save</button>
    </div>
    <div class="ce-body" data-hit="" data-nodrag="" ref=${box} style=${{ display: file && !msg ? '' : 'none' }}></div>`;
}, { cls: 'c-codeedit' });
