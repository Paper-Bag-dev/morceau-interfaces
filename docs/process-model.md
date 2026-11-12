# Process model: linked cards

Every card is its own borderless see-through window (placement, glass and stacking stay per card). What changed is how their pages run.

## Before
Each card built its own web view. On macOS WebKit gave each one its own web process: 20 cards = 21 processes, about 740 MB.

## Now (macOS)
One hidden **root page** (`ui/root.html`, `src/surface/root.rs`) opens every card with `window.open()`. WebKit keeps a page and the
windows it opens in one process, because they can reach each other's JavaScript. Measured with 20 cards on the same build:

| | web processes | memory |
|---|---|---|
| unlinked (`MORC_LINKED=0`) | +21 | +740 MB |
| linked (default) | +1 | +101 MB |

How a card is made:
1. The host builds the card's window and parks it in the root under its key.
2. The host asks the root page to `morcOpen(key)`: `window.open('index.html?k=<key>')`.
3. wry's new-window handler finds the parked window by the key in the URL and builds the card's web view on the root's configuration
   (same process, same `morc://` handler; wry does not register them twice).
4. Cards share the root's ipc channel, so `ui/core/bridge.js` prefixes each message with `k<key>\n`; the root's handler routes it.

Shared state: the root holds `window.morcShared`, a store with zustand's vanilla API (`getState`, `setState`, `subscribe`). A card reaches
the same object as `window.opener.morcShared`, no host in between.

## Rules
- A card page loads once. New content arrives by message or shared state, never by navigating (a navigation may move it to another process).
- Never declare a top-level `const ipc` in a classic page script: wry defines `window.ipc` as fixed, and the clash makes WebKit reject the
  whole script silently.
- One process means one crash takes every card; heavy work (layouts, parsing) belongs in a Web Worker so it cannot stall the others.

## Switches and platforms
- `MORC_LINKED=0` turns linking off (one process per card), for comparing or falling back.
- `MORC_PORT` runs a second host beside the live one (with its own `HOME` for its own runtime folder): how the numbers above were taken.
- Windows: off (`linked_cards()` is false); WebView2 already shares one browser process per environment. Measure before turning it on.

## Known
- Many cards shown in one burst can land on the same spot: placement runs before the earlier cards report their size. Not caused by
  linking (unlinked bursts overlap too); linked cards are faster, so it shows more. Cards added one at a time all get their own spot.

## Windows (12 Nov)
Linked cards are on by default on Windows too: the root page's WebView2 environment is handed to every card it opens
(`platform/windows/app.rs linked_builder` = `with_environment(opener.environment)`, `linked_response` = the card's ICoreWebView2), so
`window.opener.morcShared` is one object there as well. Differences handled in `surface/root.rs`: every WebView2 needs its own file
handler and ipc handler (macOS cards share the root's), and the root is built with the cards' browser flags (WebView2 allows one
configuration per browser process). Type-checked for x86_64-pc-windows-msvc; not yet run on a Windows machine.
With `MORC_LINKED=0` (either OS) the shared store is relayed by the host instead (docs/state.md), same API.
