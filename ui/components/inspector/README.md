# inspector

The editor pane of a workbench: tabs, back/forward, a breadcrumb (group / path / title) and the selected entity (`nav`): its `note`, its
`sections` of references and its `peek` (source lines with line numbers; a line's `refs` are chips). Every chip opens that entity, so you can
walk the whole graph from here.

`{type:"inspector", entities, groups, nav:"@/nav", empty:"text shown when nothing is open"}`
