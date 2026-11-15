# explorer

A grouped, searchable list of entities. A click opens the entity through `nav`, so any inspector, wire, issues or steps card bound to the same
`nav` path follows.

`{type:"explorer", title, entities, groups:[{id,label}], by:"group", only:{field:value|[values]}, folded:[groupIds], search:true, nav:"@/nav"}`

Two explorers over the same entities with different `only` filters (files vs messages) act like VS Code's activity-bar views; put them in one
card when there is no room for two.
