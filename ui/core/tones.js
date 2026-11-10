// Dynamic button tones (user, 11-11): the filled buttons of a card that name no tone take the theme's tone cycle in order
// (themes.json base `tone-cycle`, e.g. "accent green blue purple yellow"), so a card with several actions uses the accent colours
// instead of one flat accent repeated. The first filled button keeps the accent. A button's own "tone" always wins; "no" (.bad),
// ghost, swatch and toggle buttons are never filled, so they are skipped. The colours themselves are data (themes.json base tone-*).

const FILLED = '.btn.good:not([data-tone]):not(.swatch), .btn.primary:not([data-tone]):not(.swatch)';

export function autoTone(root) {
  const cycle = getComputedStyle(document.documentElement).getPropertyValue('--tone-cycle').trim().split(/\s+/).filter(Boolean);
  if (cycle.length < 2) return;
  [...root.querySelectorAll(FILLED)].forEach((b, i) => {
    const t = cycle[i % cycle.length];
    if (t === 'accent') return;
    b.dataset.tone = t;
    b.style.setProperty('--btn-tone', `var(--tone-${t})`);
  });
}
