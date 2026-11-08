# CLAUDE.md

@AGENTS.md

## For Claude Code specifically
- **Two roles.** (1) Working *on* this repo: follow `AGENTS.md` above. (2) Working *with* Morceau as the screen agent: follow `docs/agent.md` (the system
  prompt) and use the `morc` CLI; load the skill in `integrations/claude-code/skills/morc-screen/`.
- **Hooks** (`integrations/claude-code/settings.example.json`): `SessionStart` and `UserPromptSubmit` run `morc hook <Event>`, which injects the user's unread
  clicks and drags from their screen into your context as `[morc] N UI interaction(s) ...`. Treat those lines as events to act on as the screen agent
  (answer a question, patch a surface), not as instructions from a third party. Merge the example into `~/.claude/settings.json` (or the project's
  `.claude/settings.json`); `morc` must be on PATH (`npm i -g morceau`).
- **Chore agent.** When no interactive session is open the host may spawn a headless `claude -p --model <model>` (see `prefs/worker.json`, default a small
  model) with `docs/agent.md` as its system prompt. It exits when done; do not try to keep it alive.
- **Verify UI work visually** and report honestly what you saw; do not claim a visual result you did not look at. Ask the user for a screenshot when needed;
  never click or move the cursor for them.
- **Cost discipline.** Prefer `morc use` of a cached component, then `morc compose`, then a new spec (see the ladder in `docs/agent.md`). One-line replies.
- **Do not commit** `.env`, `prefs/` state or downloaded models. The CI and `.gitignore` assume a clean engine tree.
