# stat-line

Up to six short text facts in ONE low flex row, no buttons (the corner x closes it). The text counterpart that fits the THIN strips (about 100 px high)
where `stat-strip` is too tall. Layout: `cache/seed/stat-line.json`. Each fact is a ready-made string; unused slots stay empty.

**Card:** `morc card ld-4 --chip stat-line --region below --importance 0.85 --min-importance 0.85 --set t1="Cost today $4.20" --set t2="Sessions 14" --set t3="Longest reply 4m 12s"`
**Show:** `morc push ld-4`. Text stays at the floor (15 px+); to fit more, shorten the strings rather than shrinking them.
