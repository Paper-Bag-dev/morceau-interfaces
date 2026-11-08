# ask-permission

An urgent yes/no routed back to the agent (wake). Sticky, parked if ignored; `morc summon` brings it back. Layout: `cache/seed/ask-permission.json`.

**Data:** `{"title": "Delete the old logs?", "body": "12 files, 40 MB", "yes": "Delete", "no": "Keep", "ctx": "cleanup-logs"}`
**Push:** `morc use ask-permission --data @file.json`. The click answers with `{ctx, answer: yes|no}` in the inbox.
**Rule:** the question disappears after the click; never leave answered buttons on screen.
