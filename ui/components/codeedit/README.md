# codeedit

The whole file of the open entity (`nav` selection with a `file` path), in Monaco (vendored, ui/vendor/monaco): highlighting, search, multi-cursor and the VS Code keys; Ctrl+S saves, Revert drops the changes. Built on what morc has, nothing new in the engine:
- the card has `opts.persist`, so its data (`/want`, `/draft`) lands in `<home>/state/<card id>.json` about a second after a change;
- opening runs the allowlisted action `open` (default `wb-open`) into `/doc`; saving runs `save` (default `wb-save`) into `/saved`.
  Both are fixed commands in `actions.json` that read that state file; they decide which folders may be read and written.

`{type:"codeedit", entities, nav:"@/nav", open:"wb-open", save:"wb-save"}`

The canvas/editor carries `data-nodrag` (morc bridge.js): dragging inside it pans, moves nodes or selects text; the card moves only from its edges and title.
