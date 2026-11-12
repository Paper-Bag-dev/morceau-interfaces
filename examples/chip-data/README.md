# Sample data for every chip

Filled data for each chip in `cache/seed/`, for demos and load tests: `morc use <chip> --data @examples/chip-data/<chip>.json`.
The seeds themselves stay empty, so a card never shows sample rows by accident. `graph-view.json` is Morceau's own module graph
(`morc graph --root src/ --level dir`).
