# noir

Bento fintech dark skin (user, 11-12, from a Dribbble finance dashboard). Style only: every rule is keyed off `html[data-skin~="noir"]`.
Turn it on with `skin: "noir"` and accent `ice` (in `ui/themes.json` platforms, or your `prefs/ui.json` under `themes`).

| piece | look |
|---|---|
| card | solid near-black, 24px radius, no border, deep soft shadow; titles left, 17px |
| hero | a card holding one number becomes the pale-blue gradient card with dark ink and a 44px thin numeral |
| numbers | thin (300) display numerals, dim units; strips split by hairlines |
| buttons | pills: Yes / primary / tones = accent fill with dark ink; the rest = raised dark pill |
| rows | hairline dividers, no boxes; asks and pastes on a raised inner card |
| charts | straight on the card (no panel), pale-blue to slate palette |
| pin, x, layer | round dark buttons |

Colors: `ui/themes.json` `skins.noir`; sizes: `ui/tokens/noir.json`.
