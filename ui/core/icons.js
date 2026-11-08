// Tiny inline icon set (stroke icons, 24px grid). Add one = one entry here.
// Clarity rule: only `check` and `x` are read without words; every other icon-only control needs a title (src/lint.rs).

const P = {
  check:        '<path d="M20 6 9 17l-5-5"/>',
  x:            '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  plus:         '<path d="M12 5v14"/><path d="M5 12h14"/>',
  minus:        '<path d="M5 12h14"/>',
  'arrow-right':'<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>',
  'arrow-left': '<path d="M19 12H5"/><path d="m12 19-7-7 7-7"/>',
  refresh:      '<path d="M21 12a9 9 0 0 0-15.5-6.3L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 15.5 6.3L21 16"/><path d="M16 16h5v5"/>',
  copy:         '<rect x="9" y="9" width="12" height="12" rx="3"/><path d="M5 15V6a2 2 0 0 1 2-2h9"/>',
  external:     '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
  trash:        '<path d="M3 6h18"/><path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2"/><path d="m6 6 1 14a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-14"/>',
  info:         '<circle cx="12" cy="12" r="9"/><path d="M12 16v-4"/><path d="M12 8h.01"/>',
  play:         '<path d="M7 4v16l13-8z"/>',
  pause:        '<path d="M8 5v14"/><path d="M16 5v14"/>',
  bell:         '<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.9 1.9 0 0 0 3.4 0"/>',
  clock:        '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
};

export const has = (name) => name in P;

export function icon(name, size = 14) {
  const s = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  s.setAttribute('viewBox', '0 0 24 24');
  s.setAttribute('width', size);
  s.setAttribute('height', size);
  s.setAttribute('fill', 'none');
  s.setAttribute('stroke', 'currentColor');
  s.setAttribute('stroke-width', '2.6');
  s.setAttribute('stroke-linecap', 'round');
  s.setAttribute('stroke-linejoin', 'round');
  s.setAttribute('aria-hidden', 'true');
  s.innerHTML = P[name] || P.info;
  return s;
}
