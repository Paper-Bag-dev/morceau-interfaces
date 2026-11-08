# cards: remember each card, change only what changed

A **card** is one small file in your data folder, `cards/<name>.json`: `{"chip", "data", "importance", "min_importance", "region", "ttl"}`.
Write it once. After that only the data that changed is edited, with a one-line command. Never regenerate a big JSON each time.

```
morc card tok-stats --chip stat-tiles --importance 0.85 --set a.label="Tokens, all time" --set a.value=817 --set a.unit=M
morc card tok-stats --set a.value=820                 # later: change one number
morc card tok-days  --chip week-chart --set title="New tokens per day" --set values=[0.5,1.4,0.4] --set labels=["Mon","Tue","Wed"] --set unit=M
morc push tok-stats tok-days                          # show several at once; the host places them (Tier 0, see INDEX.md)
morc push --all --prefix tok-                         # or every card whose name starts with tok-
```

**Floors (what you do not compromise on):** `min_importance` is the smallest a card will be drawn (default 0.85, which keeps text above the type floor); text
size, icon size and component size do not go below it unless the user points it out. Compromise on position and on size above the floor, not below.
**Memory:** the card files are your memory of what is on screen and how big it should be. Look at `cards/` before making a new one.
