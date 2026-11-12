# resume-sessions

One-click resume of earlier Claude Code sessions. A title, then one row per session: the session title, a sub line (project, how long ago)
and a badge. A click runs that row's staged action, which opens the session in a new terminal window, and closes the card. Layout:
`cache/seed/resume-sessions.json` (component `list`, no new code).

**Stage first, one action per row:** `morc stage resume-<short id> --cwd <project dir> -- osascript -e '...do script "claude --resume <id>"'`.
Specs can only name actions; the command itself lives in actions.json.
**Data:** `{"title": "Pick up a session", "body": "", "items": [{"text": "Fix the login bug", "sub": "web-app · 2 h ago", "badge": "Resume",
"action": [{"do": "run", "id": "resume-1a2b3c4d", "into": "/out"}, {"do": "close"}]}]}`
**Push:** `morc use resume-sessions --data @file.json`. One item = the single-session card ("just this one"); five = the default list.
**Rule:** prune old staged entries (`morc stage --prune`) now and then; a row whose action was pruned shows "not in actions.json".
**Look:** rows are even one-line pills (title and sub line cut with an ellipsis, full text as the tooltip); on the glass skin they are dark lens pills.

**Actions (0.2):** `remove {id}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
