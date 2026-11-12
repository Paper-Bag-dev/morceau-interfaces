import { registerView, html, Icon, useLayoutEffect, useState } from '../../core/view.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';

// {type:"asks", items:{bind:"/items"}, ctx:"{/ctx}", send?:"Send"}   item: {id, q, yes?, no?}
// Several yes/no questions answered in any order, then ONE Send: a single wake {ctx, answers:{id: "yes"|"no"}} instead of one per question,
// so a batch of decisions costs the agent one turn. Send stays disabled until every question has an answer; the card closes after Send. Preact view.
registerView('asks', ({ c, useBind, act, host }) => {
  const raw = useBind(c.items);
  const list = Array.isArray(raw) ? raw : [];
  const [answers, setAnswers] = useState({});
  const key = JSON.stringify(list);
  useLayoutEffect(() => { if (!reduced) host.querySelectorAll(':scope > .ask').forEach((row, i) => enter(row, i * 30)); }, [key]);
  const ready = list.length > 0 && Object.keys(answers).length >= list.length;
  const pick = (id, v) => (e) => { rippleAt(e.currentTarget, e); setAnswers({ ...answers, [id]: v }); };
  const send = (e) => { if (!ready) return; rippleAt(e.currentTarget, e); act([{ do: 'wake', payload: { ctx: c.ctx, answers } }, { do: 'close' }]); };
  return html`
    ${list.map((it, i) => {
      const id = String(it.id ?? i + 1), a = answers[id];
      return html`<div key=${id} class=${'ask' + (a ? ' answered' : '')}>
        <div class="q">${it.q ?? ''}</div>
        <div class="picks flex gap-1.5">
          <button class=${'btn good pick' + (a === 'yes' ? ' on' : '')} data-hit="" onClick=${pick(id, 'yes')}><${Icon} name="check" /><span class="lb">${it.yes || 'Yes'}</span></button>
          <button class=${'btn bad pick' + (a === 'no' ? ' on' : '')} data-hit="" onClick=${pick(id, 'no')}><${Icon} name="x" /><span class="lb">${it.no || 'No'}</span></button>
        </div>
      </div>`;
    })}
    <button class="btn primary send" data-hit="" disabled=${!ready} onClick=${send}><${Icon} name="check" /><span class="lb">${c.send || 'Send answers'}</span></button>`;
}, { cls: 'c-asks', hit: false });
