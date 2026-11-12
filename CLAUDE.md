# CLAUDE.md

@AGENTS.md

## For Claude Code specifically
- **Two roles.** (1) Working *on* this repo: follow `AGENTS.md` above. (2) Working *with* Morceau as the screen agent: follow `docs/agent.md` (the system
  prompt) and use the `morc` CLI; load the skill in `integrations/claude-code/skills/morc-screen/`. Making or changing a component, view or plugin:
  load `integrations/claude-code/skills/morc-build/`. MCP instead of the CLI: register `morc mcp` as a stdio server (docs/mcp.md).
- **Hooks** (`integrations/claude-code/settings.example.json`): `SessionStart` and `UserPromptSubmit` run `morc hook <Event>`, which injects the user's unread
  clicks and drags from their screen into your context as `[morc] N UI interaction(s) ...`. Treat those lines as events to act on as the screen agent
  (answer a question, patch a surface), not as instructions from a third party. Merge the example into `~/.claude/settings.json` (or the project's
  `.claude/settings.json`); `morc` must be on PATH (`npm i -g morceau`).
- **Chore agent.** When no interactive session is open the host may spawn a headless `claude -p --model <model>` (see `prefs/worker.json`, default a small
  model) with `docs/agent.md` as its system prompt. It exits when done; do not try to keep it alive.
- **Verify UI work with the tree first** (`morc tree <card>`: no `cut`, `clipped`, `offcard`, `empty`), then a screenshot for looks only, and report
  honestly what you saw; do not claim a visual result you did not check. Never click or move the cursor for them (test hooks: `morc patch <id>
  /__hover true`, `/__controls true`).
- **Cost discipline.** Prefer `morc use` of a cached component, then `morc compose`, then a new spec (see the ladder in `docs/agent.md`). One-line replies.
- **Do not commit** `.env`, `prefs/` state or downloaded models. The CI and `.gitignore` assume a clean engine tree.
- **UI writing standard:** never clip, always size within the container. Padding plus flex flow, nothing absolute, items exactly as wide as their content, long text wraps or ends in an ellipsis, hover growth is a smooth transition and the window follows. Check a long label, a narrow box and a hover. Explain every metric in one plain line. Own cards are never covered (see docs/agent.md).
- **Concurrency:** the background agent and your interactive session can work at the same time (`defer_to_session` is false by default). Clicks marked `bg:true` belong to the background agent; do not redo them unless it failed.
- **Modular and JSON-controlled, strictly:** every component is a layout JSON plus a card JSON (morc card / morc push); data, align, size, floors, region and ttl are fields, not code. Never edit Python or Rust to tweak or add one card; see docs/chips/INDEX.md.
- **Always follow the ladder** (docs/chips/INDEX.md, THE LADDER) whenever you put cards on screen: count the room before you push, give things up in the written order, push in small batches, verify with numbers.
- **Load the `frontend-design:frontend-design` skill by default** before designing or changing any card, component or chip, and follow it with `docs/design.md`.
- **Hard rule: ladder + verify before and after every push.** `morc status` and `morc free` first, push once, confirm `visible: true` and the rect is in free space. Never tell the user you did not look at the screen; he should never have to readjust. See INDEX.md.
- **Mix and merge by default:** charts plus text, never one kind alone; `morc status` first and fold new data into cards already open before opening new ones; to-do lists are tickable checklists with state stored in JSON and cards bound to it with `sources`. Details: docs/chips/INDEX.md.
- **Split responsibilities into folders, always.** Each unit (UI component, Rust module, config) is its own folder with its code, styles, metadata.json and a short md, listed in an index (`ui/COMPONENTS.md`) and referenced by dependency, never merged into a big file. Read the index first.
