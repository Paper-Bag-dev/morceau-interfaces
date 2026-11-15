import { registerView, html, useEffect, useRef } from '../../core/view.js';

// {type:"video", src:"{/src}", title:"{/title}", w:{bind:"/vars/w"}}
// src is a YouTube link or id (played in YouTube's own embedded player) or a direct media URL (played in a <video>).
// The component owns its iframe; the generic `el` still never makes one. Size: tokens vd-w / vd-h (ui/tokens/video.json).
const ytId = (s) => {
  if (/^[\w-]{11}$/.test(s)) return s;
  const m = String(s).match(/(?:youtu\.be\/|[?&]v=|\/embed\/|\/shorts\/)([\w-]{11})/);
  return m ? m[1] : null;
};

registerView('video', ({ c, useBind, host, act }) => {
  const src = String(useBind(c.src) || '');
  const title = useBind(c.title) || 'Video';
  const id = ytId(src);
  const w = useBind(c.w);   // optional width from the card's /vars (expand grows it); the token vd-w otherwise
  host.title = title;
  // Playback climbs the tree: component -> card data /playback (kept by the card, seen in `morc tree`) -> shared @/media
  // (any card can bind it) -> the agent: playing / paused as an inbox row (emit, no wake), ended as ONE wake per video.
  // Sources: YouTube's player messages (enablejsapi) or the <video> events.
  const frame = useRef(null), last = useRef('');
  const report = (state) => {
    const key = src + '|' + state;
    if (last.current === key) return;
    last.current = key;
    const at = Date.now(), info = { state, title, src, at };
    const steps = [{ do: 'set', path: '/playback', value: info }, { do: 'set', path: '@/media', value: info }];
    steps.push(state === 'ended' ? { do: 'wake', payload: { ask: 'video_ended', title, src } } : { do: 'emit', payload: { kind: 'video_' + state, title, src } });
    act(steps);
  };
  const ended = () => report('ended');
  const YT = { 0: 'ended', 1: 'playing', 2: 'paused' };
  useEffect(() => {
    if (!id) return undefined;
    const listen = () => { try { frame.current && frame.current.contentWindow.postMessage(JSON.stringify({ event: 'listening', id: 1, channel: 'widget' }), '*'); } catch (_) { /* not loaded yet */ } };
    const onMsg = (e) => {
      if (!/youtube(-nocookie)?\.com$/.test(new URL(e.origin).hostname) || e.source !== frame.current?.contentWindow) return;
      let m; try { m = typeof e.data === 'string' ? JSON.parse(e.data) : e.data; } catch (_) { return; }
      const st = m && (m.event === 'onStateChange' ? m.info : m.info && m.info.playerState);
      if (YT[st]) report(YT[st]);
    };
    window.addEventListener('message', onMsg);
    const t = setInterval(listen, 1500); listen();
    return () => { window.removeEventListener('message', onMsg); clearInterval(t); };
  }, [id, src]);
  host.style.width = w ? (typeof w === 'number' ? w + 'px' : w) : '';
  const body = id
    ? html`<iframe class="vd-frame" src=${`https://www.youtube-nocookie.com/embed/${id}?autoplay=1&rel=0&modestbranding=1&enablejsapi=1`} ref=${frame} onLoad=${() => setTimeout(() => frame.current && frame.current.contentWindow.postMessage(JSON.stringify({ event: 'listening', id: 1, channel: 'widget' }), '*'), 300)}
        title=${title} allow="autoplay; encrypted-media; picture-in-picture; fullscreen" allowfullscreen
        referrerpolicy="strict-origin-when-cross-origin"></iframe>`
    : html`<video class="vd-frame" src=${src} controls autoplay playsinline onEnded=${ended} onPlay=${() => report('playing')} onPause=${(e) => !e.currentTarget.ended && report('paused')}></video>`;
  return html`<div class="vd-title">${title}</div><div class="vd-body" data-hit>${body}</div>`;
}, { cls: 'c-video' });
