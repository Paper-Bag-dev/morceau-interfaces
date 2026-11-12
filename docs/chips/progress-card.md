# progress-card

Loading or download status: a title and up to three progress bars in percent. Layout: `cache/seed/progress-card.json`. Use it to pass along what another app is doing
(browser downloads, a Windows update, a build) while it happens. A row with an empty label is hidden.

**Card:** `morc card dl --chip progress-card --ttl 120 --set title="Chrome downloads" --set a.label="paper.pdf" --set a.value=12 --set b.label="dataset.zip" --set b.value=60`
**Show:** `morc push dl`. **Update live, no re-push:** `morc patch dl /a/value 45` (the bar animates). When it reaches 100, `morc patch dl /a/label "paper.pdf: done"`, then let the `ttl` close it.
**Where the numbers come from:** a source you can read (a download folder, a log, the Chrome extension's status); until that is wired, pretend values are fine for tests, and say so.

**Actions (0.2):** `set {key: a|b|c, value}`, `label {key, text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
