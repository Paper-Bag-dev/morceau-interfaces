# fluent

The Windows 11 (Fluent, dark) skin. Style only: every rule is keyed off `html[data-skin~="fluent"]`, so it changes nothing until a theme
names the skin. Windows names it by default (`ui/themes.json` `platforms.win`: `skin: "fluent"`, `default: "system"`).

| piece | look |
|---|---|
| plate | acrylic from Windows underneath (`src/platform/windows/glass.rs`), a light tint, a hairline stroke, 8px corners, flyout shadow |
| buttons | accent = filled with your Windows accent and dark text; standard = quiet fill with a stroke, white text; 4px corners, 32px tall |
| rows, asks, pastes, checklist | card rows: soft fill, faint stroke, lift on hover; check boxes are 4px squares filled with the accent when done |
| badges, progress | subtle 4px chips; progress is a hairline track with a 3px accent bar |
| pin, x, layer | caption-style 4px squares; the x turns red on hover |

Colors: `ui/themes.json` `skins.fluent` and `accents.system` (the host reads the Windows accent). Sizes: `ui/tokens/fluent.json`.
