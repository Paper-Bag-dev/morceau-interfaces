// Component catalog. A spec may only use types registered here: the whitelist is the security model.
// Add a component = one folder ui/components/<name>/ (code, styles, metadata.json) that calls register(); morc ui build lists it in ui/components.json,

// which core/main.js loads in dependency order. Index of everything: ui/COMPONENTS.md.

const catalog = new Map();

/** fn(spec, ctx) -> HTMLElement. Mark clickable/visible boxes with data-hit so the window shape follows them. */
export function register(type, fn) {
  catalog.set(type, fn);
}

export const get = (type) => catalog.get(type);
export const types = () => [...catalog.keys()];
