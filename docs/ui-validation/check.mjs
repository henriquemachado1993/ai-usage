import { writeFile } from 'node:fs/promises';

const tabs = await fetch('http://127.0.0.1:9333/json').then(r => r.json());
const tab = tabs.find(t => t.type === 'page');
const socket = new WebSocket(tab.webSocketDebuggerUrl);
await new Promise(resolve => socket.addEventListener('open', resolve, { once: true }));
let id = 0;
const pending = new Map();
socket.addEventListener('message', e => {
  const message = JSON.parse(e.data);
  if (!message.id) return;
  const { resolve, reject } = pending.get(message.id);
  pending.delete(message.id);
  message.error ? reject(new Error(message.error.message)) : resolve(message.result);
});
const command = (method, params = {}) => new Promise((resolve, reject) => {
  pending.set(++id, { resolve, reject });
  socket.send(JSON.stringify({ id, method, params }));
});
const evaluate = async expression => {
  const result = await command('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
};
const render = () => evaluate('new Promise(r => setTimeout(() => requestAnimationFrame(r), 100))');
const assert = async (expression, name) => {
  if (!await evaluate(expression)) {
    console.log(await evaluate(`({ url:location.href, body:document.body.innerText, html:document.querySelector('#root')?.innerHTML.slice(0,500) })`));
    throw new Error(name);
  }
  console.log(`OK: ${name}`);
};
const capture = async path => {
  const { data } = await command('Page.captureScreenshot', { format: 'png' });
  await writeFile(new URL(path, import.meta.url), Buffer.from(data, 'base64'));
};
const navigate = async url => {
  await command('Page.navigate', { url });
  await new Promise(resolve => setTimeout(resolve, 1000));
  for (let attempt = 0; attempt < 20; attempt++) {
    if (await evaluate(`!!document.querySelector('.widget,.strip')`)) return;
    await new Promise(resolve => setTimeout(resolve, 250));
  }
};
await command('Emulation.setDeviceMetricsOverride', { width: 390, height: 650, deviceScaleFactor: 1, mobile: false });
await navigate('http://127.0.0.1:1420/');
await evaluate(`localStorage.removeItem('ai-usage:mock:scale')`);
await navigate('http://127.0.0.1:1420/');
await assert(`document.querySelectorAll('.row').length === 5`, 'App usa os dados do IPC simulado');
await capture('expanded-100.png');
await evaluate(`document.querySelector('[title="Configurações"]').click()`);
await render();
const changeScale = async value => {
  await evaluate(`{ const control = [...document.querySelectorAll('label')].find(e => e.textContent.includes('Tamanho do widget')).querySelector('select'); control.value = '${value}'; control.dispatchEvent(new Event('change', {bubbles:true})); }`);
  await render();
};
await changeScale(85);
await assert(`Math.abs(document.querySelector('.widget').getBoundingClientRect().width - 289) < 1`, '85% reduz o widget completo');
await capture('settings-85.png');
await navigate('http://127.0.0.1:1420/?collapsed&edge=right');
await assert(`Math.abs(document.querySelector('.strip').getBoundingClientRect().width - 37.4) < 1`, 'Faixa compacta de 44px respeita escala persistida');
await capture('compact-85.png');
await evaluate(`document.querySelectorAll('.strip-item')[2].click()`);
await render();
await assert(`document.querySelectorAll('.row')[2].querySelector('.detail') !== null`, 'Clique em conta compacta abre os detalhes certos');
await assert(`document.querySelectorAll('.row-head')[2].getAttribute('aria-expanded') === 'true'`, 'Detalhes acessíveis por botão');
await evaluate(`document.querySelector('[title="Configurações"]').click()`);
await render();
await changeScale(70);
await evaluate(`document.querySelector('[title="Recolher para a borda"]').click()`);
await render();
await assert(`Math.abs(document.querySelector('.strip').getBoundingClientRect().width - 30.8) < 1`, 'Compacto a 70%');
await capture('compact-70.png');
await evaluate(`document.querySelector('.strip-toggle').click()`);
await render();
await evaluate(`document.querySelector('[title="Configurações"]').click()`);
await render();
await changeScale(115);
await capture('settings-115.png');
await evaluate(`document.querySelector('[title="Recolher para a borda"]').click()`);
await render();
await assert(`[...document.querySelectorAll('.strip-item,.strip-toggle')].every(e => {const r=e.getBoundingClientRect();const p=e.closest('.strip').getBoundingClientRect();return r.left>=p.left&&r.right<=p.right&&r.bottom<=p.bottom;})`, 'Botões cabem na faixa a 115%');
await capture('compact-115.png');
await evaluate(`localStorage.removeItem('ai-usage:mock:scale')`);
await command('Browser.close');
socket.close();
