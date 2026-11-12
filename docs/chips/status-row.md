# status-row

A thin row for the strip above or below the active window: one spinner and up to two progress bars, no buttons (the corner x closes it). Use it to show what
another app is doing (browser downloads, a Windows update, a build) without taking a side. Layout: `cache/seed/status-row.json`.

**Card:** `morc card dl --chip status-row --region below --align left --ttl 300 --set s="Waiting for the Windows update" --set a.label="paper.pdf" --set a.value=40 --set b.label="data.zip" --set b.value=75`
**Show:** `morc push dl`. **Live update (no re-push):** `morc patch dl /a/value 65`; when done `morc patch dl /s ""` hides the spinner.
**Size:** about 60 px of visible height, so it fits the thin strips (about 117 px) where no other card does. For three or more bars or a Close button use `progress-card`.

**Actions (0.2):** `status {text}`, `set {key: a|b, value}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
