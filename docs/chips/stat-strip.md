# stat-strip

Four big numbers in ONE flex row, no buttons (the corner x closes it): a long, low card for the free strip above or below the active window. Layout: `cache/seed/stat-strip.json`.
It sets `max_frac` 0.9 so the row does not wrap into a column; that is how a chip says "let me be wide".

**Card:** `morc card tok-stats --chip stat-strip --region above --importance 0.8 --min-importance 0.7 --set a.label="Tokens, all time" --set a.value=818 --set a.unit=M ...` (b, c, d the same).
**Show:** `morc push tok-stats`. Use `--region below` for the strip under the window. Text cards go in strips as rows; pictures go in the tall sides.
