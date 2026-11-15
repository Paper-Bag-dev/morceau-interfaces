# nav

The navigation model every workbench part shares. No component of its own: `explorer`, `inspector`, `wire`, `issues` and `steps` import it.

- `useNav(props)`: `{sel, tabs, back, fwd, open(id), close(id), back(), fwd()}` stored at the spec's `nav` path (default `@/nav`, the shared store),
  so separate cards move together. Two apps on screen use two paths (`"nav": "@/nav_code"`).
- `useEntities(props)`: the spec's `entities` (list or `{bind}`) indexed by id.
- `Ref`, `Dot`: the reference chip and status dot, one look everywhere.

Entity schema: `{id, title, sub?, group?, kind?, status?: ok|warn|bad, note?, sections?: [{title, refs}], from?: [{ref, n?}], to?: [{ref, n?}], peek?: [{n, t, refs?}]}`.
