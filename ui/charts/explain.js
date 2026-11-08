// "How to read this" for chart types you might not know. Shown at most twice, then never again once you say Got it.
// `plain` is the one-line fallback if it didn't land. Update these when friction shows a wording doesn't work.
export const EXPLAIN = {
  clock:   { short: 'The dial is a 24-hour day (top = midnight, right = 6 AM). Each wedge is one hour; the further it reaches out, the more happened then.', plain: 'Longest wedge = your busiest hour.' },
  rings:   { short: 'Each ring is one goal. A closed circle means you hit the target; a bright second lap means you went past it.', plain: 'Closed ring = target reached.' },
  treemap: { short: 'Box size is share of the total: bigger box, bigger chunk. Like a pie chart, but the names fit.', plain: 'Biggest box = biggest chunk.' },
  radar:   { short: 'Each spoke is a different measure, the center is zero and the edge is the max. A round, full shape means balanced; a lopsided one shows what is missing.', plain: 'Even shape = balanced week.' },
  graph:   { short: 'Each dot is a file; a line means one uses the other. Dots of one color sit together as a layer. The orange dashed route is the path your click travels, from the button to the screen and back.', plain: 'Dots are files, lines are who uses whom.' },
  heatmap: { short: 'Each square is one slot (e.g. a day). Brighter = more. Look for bright streaks or empty gaps.', plain: 'Bright = a lot, dark = none.' },
};

export const FANCY = new Set(Object.keys(EXPLAIN));

