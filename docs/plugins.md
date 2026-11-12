# Plugins and reload

| kind | lives | loads | reload |
|---|---|---|---|
| component | `ui/components/<name>/` (repo) | listed in `ui/components.json` (`morc ui build`) | `morc refresh <name>` |
| repo plugin | `ui/plugins/*.js` | every card | `morc refresh <file>` |
| **your plugin** | `<home>/plugins/*.js` (outside the repo) | every card, after the repo's (served at `user-plugins/`) | `morc refresh <file or name>` |
| CLI plugin | `tools/cli/<name>.py` | `morc <name>` | nothing to restart (runs per call) |
| chip | `cache/` JSON | `morc use <name>` | `morc save` |

A plugin is an ES module that registers itself: `register(type, fn)` (vanilla) or `registerView(type, Component)` (Preact, docs/views.md).
From `<home>/plugins` import engine modules with `../core/<file>.js`.

`morc refresh [names...]` re-imports the named components / plugins (all when no name) in every open card under a new URL, swaps their
stylesheets and rebuilds the card from its store: **the data stays, the card does not pop again**. What only the DOM held (scroll,
focus) is rebuilt. Errors go to host.log as `refresh <name>: ...`.
