# video-card

A 16:9 video player on the plate (component `video`, `ui/components/video/`). A YouTube link or id plays in YouTube's embedded player; any other URL plays in a `<video>` with controls. Width is the token `vd-w` (`ui/tokens/video.json`). Layout: `cache/seed/video-card.json`.

**Data:** `{"title": "Bad Apple!!", "src": "https://www.youtube.com/watch?v=FtutLA63Cp8"}`
**Push:** `morc use video-card --id <id> --data @file.json --region auto --force`.

**Actions:** `play {src, title}` swaps the video in place (`morc do <id> play '{...}'`).
**Expand:** the seed's `opts.expand_vars` `{"w": 1150}` widens the player on expand; `morc expand watch` applies it (and clears it on shrink).
**Shrink (11-15):** patching `/vars/w` back is not enough: the card's width stays locked at the expanded size. Also `morc card <id> size --frac <pre-expand frac, else 0.6>` (a new cap unlocks and re-measures), then `move --at` the pre-expand spot, clamped on screen.

**Ended hook (11-15):** when the video finishes the card sends a wake `{ask: "video_ended", title, src}` (YouTube player messages via enablejsapi, or the `<video>` ended event), once per video.
**Playback (11-15):** state climbs the tree: card data `/playback` {state, title, src, at} (persisted, in `morc tree`), shared `@/media` (bind it from any card), then the agent: playing/paused as `emit` rows (`video_playing`, `video_paused`), ended as the one wake.
