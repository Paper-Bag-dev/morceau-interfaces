# sprint-timer

A small ring timer that stays on screen for a work sprint: the arc fills toward the goal, the elapsed time sits inside it
(seconds under it), the end time (or minutes over) below, and one pause / resume icon. Component `timer` (`ui/components/timer/`), sizes in `ui/tokens/timer.json`. Sticky and `persist`, so it survives a host restart with its state.

**Data:** `{"title": "GPT from scratch", "start": <epoch ms>, "goal_min": 240}` (`paused_at`, `paused_ms` are written by the button).
**Push:** `morc use sprint-timer --id timer-<name> --data @file.json --force` (place it by the ladder: top strip or a free corner).
**Actions:** `morc do <id> goal '{"min":300}'` changes the goal; `morc do <id> restart` starts again from now.
Rests at 35% opacity until hovered (tokens tm-rest-opacity, tm-fade-ms).
