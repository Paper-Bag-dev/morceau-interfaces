// gauge: one number against a limit (video minutes vs budget). Arc sweeps, a needle swings with overshoot,
// and the color shifts good -> warn -> bad as it fills.
import { easeOutBack, clamp01, label, fmt, ink } from './util.js';

// good / warn / bad come from the theme (themes.json base good, warn, bad), so a gauge follows the skin like everything else
const TONE_VARS = ['--good', '--warn', '--bad'], TONE_FALLBACK = ['142 60% 50%', '38 92% 55%', '0 80% 62%'];
const tones = () => { const cs = getComputedStyle(document.getElementById('stage')); return TONE_VARS.map((v, i) => cs.getPropertyValue(v).trim() || TONE_FALLBACK[i]); };

export function draw(g, s) {
  const max = s.c.max || 100, val = s.values[0] ?? 0, ratio = clamp01(val / max);
  const cx = s.W / 2, cy = s.H - 18, R = Math.min(s.W / 2 - 14, s.H - 30), a0 = Math.PI, a1 = Math.PI * 2;
  const shownRatio = ratio * clamp01(s.t);
  const TONES = tones();
  const tone = ratio >= (s.c.dangerAt ?? 1) ? TONES[2] : ratio >= (s.c.warnAt ?? 0.8) ? TONES[1] : TONES[0];

  g.lineCap = 'round';
  g.lineWidth = 12; g.strokeStyle = ink(.1);
  g.beginPath(); g.arc(cx, cy, R, a0, a1); g.stroke();
  g.strokeStyle = `hsl(${tone})`; g.shadowColor = `hsl(${tone} / .6)`; g.shadowBlur = 14;
  g.beginPath(); g.arc(cx, cy, R, a0, a0 + (a1 - a0) * shownRatio); g.stroke(); g.shadowBlur = 0;

  // needle (overshoots then settles)
  const na = a0 + (a1 - a0) * clamp01(ratio * easeOutBack(clamp01(s.t)));
  g.strokeStyle = ink(1); g.lineWidth = 2.5;
  g.beginPath(); g.moveTo(cx + Math.cos(na) * (R - 24), cy + Math.sin(na) * (R - 24)); g.lineTo(cx + Math.cos(na) * (R - 6), cy + Math.sin(na) * (R - 6)); g.stroke();

  label(g, fmt(val * clamp01(s.t), s.unit), cx, cy - R * 0.38, { size: 26, weight: 600, color: ink(1), display: true });
  label(g, `of ${max}${s.unit || ''}`, cx, cy - R * 0.38 + 20, { size: 11, color: ink(.6) });
}

export const hit = () => -1;

