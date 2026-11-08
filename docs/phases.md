# Phases

## Phase 1: built (needs your verification pass)
Rust host, no-focus transparent windows shaped to content, free-space placement that avoids the active window and glides away when
you switch windows, expiry + hover/interact/pin/drag rules, springs/ripple/shimmer/stagger, rotating accent themes, component cache +
registry, live data sources, anticipation triggers with engagement back-off, agent inbox + hooks + headless wake, 11 chart kinds with a
teaching loop, an optional behaviour log other tools can read, CLI + agent skill.

Verified so far (by me): builds clean, 4 unit tests (lifetime), a real window rendered transparent over other windows, a real click
travelled page -> host -> literacy file -> inbox, chart swapped to the plain version. NOT yet verified: drag-by-hold, pin button,
text-input keyboard focus, the hooks firing in a fresh session, headless wake (dry-run only), triggers firing on real data (shipped disabled).

## Phase 2: sees, shows and summons
- **Summon anywhere:** tray icon + global hotkey (Ctrl+Alt+Space) that pulls every parked/urgent surface next to the active window; Windows
  toast + ntfy phone push for urgent asks while you are away from the screen.
- **Tours:** on-device Windows UI Automation to find the exact button/field (text, role, bounds), draw a ring + callout on it, advance
  when the control is used or the window changes. Local vision model on the GPU PC for apps UIA cannot read (games, canvases). Screen
  capture only on demand, never stored, never sent out.
- **3D:** three.js with a transparent canvas, `.glb` from Blender, orbit/zoom/click-select; window is just the model, no frame.
- **More charts:** sankey (where time flows), streamgraph, bullet, slope, calendar streaks.
- **Follow-me:** reposition on window move/resize (location-change hook, throttled), per-monitor DPI edge cases.
- **Hub link:** the same protocol over a small hub so a second machine can serve when the first is off, and phone surfaces share the vocabulary.
- **Safer actions:** per-action confirm for anything destructive; action allowlist editor.

## Last phase: polish from real data (after a couple of weeks of use)
- Tune the show-vs-silence balance per topic from engagement.jsonl: default TTLs, cooldown curve, time-of-day windows, which topics earn auto pushes.
- Placement taste: which side/anchor you actually leave alone vs dismiss; spacing and sizes you hover on.
- Motion taste: keep the animations you linger on, cut the ones followed by instant dismissal; reduced-motion and quiet-hours modes.
- Chart literacy: which kinds you got, which stayed confusing, rewrite explainers, retire kinds that never land.
- Theme taste: which accents correlate with engagement.
- Performance: measure idle/active RAM and CPU, trim the warm-window policy; startup at login.
- Failure sweep: sleep/resume, monitor hot-plug, RDP, fullscreen apps and games (Vanguard), high-DPI mixes.

## Status update (11-08, later)
Also verified by me with real screenshots: backdrop-aware colors flipped correctly over a dark editor; `fill` scaled a chart to ~17% of the
screen (cap is 25%); rounded shapes; text outline over a split white/dark backdrop; the pop-in no longer locks a wrong window shape.
Fixed after your screenshot: the window shape was measured mid-animation (clipped, curvy edges); a native Windows caption close box appeared
on click (my DWM change made it worse; now all caption/frame style bits are stripped). **Still needs your eyes:** click the window and
confirm there is no border or X; drag-by-hold; pin; typing in an input; hooks in a fresh session.
