// Component catalog. A spec may only use types registered here: the whitelist is the security model.
// Add a component = one file in ui/components/ that calls register() and one import line in core/main.js.

const catalog = new Map();

/** fn(spec, ctx) -> HTMLElement. Mark clickable/visible boxes with data-hit so the window shape follows them. */
export function register(type, fn) {
  catalog.set(type, fn);
}

export const get = (type) => catalog.get(type);
export const types = () => [...catalog.keys()];
