# todo-list (the `checklist` component)

A tickable to-do list: a count and a hairline fill, round checkboxes, clock-time badges, a coral ring on what is due. Replaces any static list of tasks.

**Spec shape:** one component `{"id":"root","type":"checklist","title":"{/title}","items":{"bind":"/items"},"run":"tick-"}`, data `{title, items:[{id,text,sub?,badge?,done}]}`.
**Making it live (the rule):** keep the state in a JSON file and bind to it: `"sources": {"/items": {"file": "<json>", "pointer": "/todo", "every": 2}}`. A click flips the row at once, then runs the allowlisted action `tick-<id>` (add one entry per item to `actions.json`) that rewrites that JSON; the next poll shows the same thing, so the file is always the truth.
**Example wiring:** a small CLI plugin in `tools/cli/` builds the spec from your JSON file and shows it with `morc show -`; a tick script (named in `actions.json`) flips `done` for one item and rewrites the file, and can also call your task manager's CLI.
**Merge:** if a checklist is already open, update its JSON; do not open a second one.
