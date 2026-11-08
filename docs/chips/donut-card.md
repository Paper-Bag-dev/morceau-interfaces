# donut-card

One donut chart (share of a whole) with a title and Close. Layout: `cache/seed/donut-card.json`. Values are in millions (M).

**Data:** `{"title": "Tokens by model", "values": [370, 281, 146, 9], "labels": ["Sonnet 5.5", "Opus 5.5", "Opus 4.8", "Haiku 5.5"]}`
**Push:** `morc use donut-card --id <id> --data @file.json --region auto`. Labels 12 characters or fewer, at most 7 slices.
**Size:** default `--importance 1.0`; 0.8 smaller, 1.2 bigger.
