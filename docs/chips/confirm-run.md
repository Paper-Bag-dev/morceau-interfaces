# confirm-run

Ask once, then the button runs a prepared command on the host: no model wakes, zero tokens. The output shows in the card. Layout:
`cache/seed/confirm-run.json`.

**Stage first:** `morc stage <id> [--cwd DIR] -- <argv...>` adds `<id>` to actions.json (marked staged). Prepare everything the command needs
(a message file, a script) before staging. `morc stage --prune` clears old staged entries.
**Data:** `{"title": "Commit the fix?", "body": "2 files, local only", "run": "<id>", "yes": "Commit", "no": "Not now", "ctx": "commit-fix"}`
**Push:** `morc use confirm-run --data @file.json`. Not now writes `{ctx, answer:"no"}` to the inbox without waking anyone.
**Rule:** only commands whose result needs no judgment. Never stage a push, a delete or anything that touches someone else.
