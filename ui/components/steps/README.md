# steps

An ordered route through entities (a user flow, a request path) on a rail. The open step lights, the ones before it fill; previous/next walk
the route through `nav`, so the inspector and wire follow each step. Steps that are not ok show their dot.

`{type:"steps", entities, route:[ids] or {bind}, nav:"@/nav", title}`
