import { registerView, html } from '../../core/view.js';

// {type:"spinner", label:"Reading your last 7 days"}: shown while something is being built; pair it with `when` on a stage value
// so it disappears the moment the result arrives. The ring is the only always-moving thing on screen, so it means "working". Preact view.
registerView('spinner', ({ c, useBind }) => html`<span class="ring shrink-0"></span><span class="lb min-w-0 truncate">${useBind(c.label) ?? ''}</span>`,
  { cls: 'spinner plate' });
