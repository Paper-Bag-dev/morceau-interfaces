# skeleton-card

An empty shimmering placeholder popup. Layout: `cache/seed/skeleton-card.json`. No data.

**Use:** when a result will open in a NEW popup, show this first under the id the real card will use:
`morc use skeleton-card --id <id-of-the-real-card> --region right --ttl 40`, then build the real card with the same id; it replaces the placeholder in place.
Skeletons inside a card are only for content landing in that same card. If the real card never comes, the placeholder expires on its own (ttl).
