// What the user has learned to read. The host persists this (literacy.json); the page just mirrors and reports.
// Per chart kind: seen (explainer shown), got (tapped "Got it"), confused (tapped "Still confusing").
import { send } from './bridge.js';

let map = {};

export const set = (m) => { map = m && typeof m === 'object' ? m : {}; };
export const get = (kind) => map[kind] || { seen: 0, got: 0, confused: 0 };

export function mark(kind, v) {
  const e = { ...get(kind) };
  if (v === 'seen') e.seen++;
  if (v === 'got') e.got++;
  if (v === 'confused') e.confused++;
  map[kind] = e;
  send({ t: 'literacy', kind, v });
}
