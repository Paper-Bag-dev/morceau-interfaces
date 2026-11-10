import { register } from '../../core/registry.js';
import { enter, reduced } from '../../core/motion.js';
import { rippleAt } from '../../core/ripple.js';
import { icon } from '../../core/icons.js';

// {type:"asks", items:{bind:"/items"}, ctx:"{/ctx}", send?:"Send"}   item: {id, q, yes?, no?}
// Several yes/no questions answered in any order, then ONE Send: a single wake {ctx, answers:{id: "yes"|"no"}} instead of one per question,
// so a batch of decisions costs the agent one turn. Send stays disabled until every question has an answer; the card closes after Send.
register('asks', (c, ctx) => {
  const el = document.createElement('div');
  el.className = 'c-asks';
  const answers = {};
  const send = document.createElement('button');
  send.className = 'btn primary send';
  send.dataset.hit = '';
  send.append(icon('check', 13));
  const sl = document.createElement('span');
  sl.className = 'lb';
  sl.textContent = c.send || 'Send answers';
  send.append(sl);
  let total = 0;
  const refresh = () => { send.disabled = Object.keys(answers).length < total; };

  ctx.bind(c.items, (items) => {
    el.replaceChildren();
    const list = Array.isArray(items) ? items : [];
    total = list.length;
    list.forEach((it, i) => {
      const id = String(it.id ?? i + 1);
      const row = document.createElement('div');
      row.className = 'ask';
      const q = document.createElement('div');
      q.className = 'q';
      q.textContent = it.q ?? '';
      const pick = (v, label, kind) => {
        const b = document.createElement('button');
        b.className = `btn ${kind} pick`;
        b.dataset.hit = '';
        b.append(icon(v === 'yes' ? 'check' : 'x', 13));
        const lb = document.createElement('span');
        lb.className = 'lb';
        lb.textContent = label;
        b.append(lb);
        b.addEventListener('click', (e) => {
          rippleAt(b, e);
          answers[id] = v;
          row.querySelectorAll('.pick').forEach((x) => x.classList.toggle('on', x === b));
          row.classList.add('answered');
          refresh();
        });
        return b;
      };
      const btns = document.createElement('div');
      btns.className = 'picks';
      btns.append(pick('yes', it.yes || 'Yes', 'good'), pick('no', it.no || 'No', 'bad'));
      row.append(q, btns);
      el.append(row);
      if (!reduced) enter(row, i * 30);
    });
    el.append(send);
    refresh();
  });
  send.addEventListener('click', (e) => {
    if (send.disabled) return;
    rippleAt(send, e);
    ctx.act([{ do: 'wake', payload: { ctx: c.ctx, answers } }, { do: 'close' }]);
  });
  return el;
});
