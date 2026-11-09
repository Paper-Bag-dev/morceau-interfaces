# checklist

A tickable to-do list on one plate. Type `checklist`, props: `title`, `items` ({bind:"/items"}, each `{id, text, sub?, badge?, done}`), `run` (action prefix, default `tick-`), `max` (default 12).

- Header: a count ("3 of 9") in the display face with a coral number, and a 3 px fill line under it that springs to the new ratio.
- Row: round checkbox, text, optional sub line, badge. A click (or Space/Enter) flips `done` in the model at once, draws the tick, strikes the text and hides the sub line; then it runs the host action `<run><id>` from `actions.json`.
- An open row whose badge is a clock time that has passed gets a coral ring and a coral badge ("due"). Refreshes every minute.
- State lives in a JSON file: bind `items` to it with the spec's `sources` so the file stays the truth (see `docs/chips/todo-list.md`).
- Reduced motion turns every transition off. Sizes are tokens in `ui/tokens/checklist.json`.
