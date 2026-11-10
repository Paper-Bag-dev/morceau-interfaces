// plate: the light on the macOS glass follows the pointer a little (--mx/--my on each card plate; plate.css eases them), then drifts back
// when the pointer leaves. Style only: nothing else reads these.
// runs for every skin; only the glass skin's CSS draws the sheen, so elsewhere the variables are simply unused
{
  const rest = (el) => { el.style.setProperty('--mx', '28%'); el.style.setProperty('--my', '-80%'); };
  document.addEventListener('pointermove', (e) => {
    for (const el of document.querySelectorAll('.card-plate')) {
      const r = el.getBoundingClientRect();
      if (!r.width || !r.height) continue;
      // keep it subtle: the light moves within the middle of the plate, never to the far edges
      const x = 20 + 60 * Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
      const y = -10 + 50 * Math.min(1, Math.max(0, (e.clientY - r.top) / r.height));
      el.style.setProperty('--mx', x.toFixed(1) + '%');
      el.style.setProperty('--my', y.toFixed(1) + '%');
    }
  }, { passive: true });
  document.addEventListener('pointerleave', () => document.querySelectorAll('.card-plate').forEach(rest));
}
