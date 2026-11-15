# skyline-card

Today as a 2D city (component `skyline`): one building per hour since the day began, as tall as its minutes, floors lit by study, video and games, quiet hours dark, a sun or moon over the current hour. Hover a building for its minutes. Live: bind `/day` with `sources` to any per-hour JSON (`{hours:[{h, <cat>: min}], now_h}`); `cats` sets the categories and labels.

**Push:** `morc use skyline-card --id skyline --region auto`.
**Music:** `morc beat` (system audio) or `morc beat --demo` feeds `@shared /beat`; buildings jump with their band, windows cycle RGB, and the `beat` plugin (`ui/plugins/beat.js`) gives every card an RGB glow (`--dance` also moves the windows). `morc beat --off` stops it.
