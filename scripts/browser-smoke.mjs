// Optional real-browser check using Chrome/Edge's DevTools protocol; no npm dependencies.
// Run the local server first. CHROME_PATH can select another Chromium executable.
import { spawn } from 'node:child_process';
import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { once } from 'node:events';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const origin = process.env.WEB_URL ?? 'http://127.0.0.1:8080';
const browserPath = process.env.CHROME_PATH ?? (process.platform === 'win32'
  ? 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe' : 'chromium');
const profile = path.join(root, 'target', `web-smoke-${Date.now()}`);
await mkdir(profile, { recursive: true });
const browser = spawn(browserPath, [
  '--headless=new', '--no-first-run', '--no-default-browser-check', '--disable-background-networking',
  '--enable-unsafe-swiftshader', '--use-angle=swiftshader', '--mute-audio',
  '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank',
], { windowsHide: true, stdio: 'ignore' });
browser.on('error', (error) => { console.error(error.message); process.exitCode = 1; });
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function until(check, timeout = 30000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    const result = await check();
    if (result) return result;
    await delay(100);
  }
  throw new Error('Browser check timed out');
}
let ws;
try {
  const portFile = path.join(profile, 'DevToolsActivePort');
  await until(async () => { try { await access(portFile); return true; } catch { return false; } });
  const port = (await readFile(portFile, 'utf8')).split('\n')[0];
  const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  ws = new WebSocket(pages.find((page) => page.type === 'page').webSocketDebuggerUrl);
  await once(ws, 'open');
  let nextId = 0;
  const pending = new Map();
  const errors = [];
  ws.addEventListener('message', ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) {
      const task = pending.get(message.id);
      if (task) { pending.delete(message.id); message.error ? task.reject(message.error) : task.resolve(message.result); }
    } else if (message.method === 'Runtime.exceptionThrown') {
      errors.push(message.params.exceptionDetails.exception?.description ?? message.params.exceptionDetails.text);
    } else if (message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error') {
      errors.push(message.params.args.map((arg) => arg.value ?? arg.description).join(' '));
    }
  });
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const id = ++nextId;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
  const evaluate = async (expression) => {
    const result = await send('Runtime.evaluate', { expression, returnByValue: true, userGesture: true });
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
    return result.result.value;
  };
  await send('Runtime.enable');
  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', { width: 1200, height: 900, deviceScaleFactor: 1, mobile: false });
  await send('Page.addScriptToEvaluateOnNewDocument', { source: `
    window.__frameCosts = []; window.__frameIntervals = []; window.__recordFrames = false;
    const originalRAF = window.requestAnimationFrame.bind(window);
    let previousFrame = 0;
    window.requestAnimationFrame = callback => originalRAF(timestamp => {
      const before = performance.now();
      callback(timestamp);
      if (window.__recordFrames) {
        window.__frameCosts.push(performance.now() - before);
        if (previousFrame) window.__frameIntervals.push(timestamp - previousFrame);
        previousFrame = timestamp;
      } else previousFrame = 0;
    });
    window.__audioContexts = []; window.__audioGains = []; window.__audioStarts = 0;
    const OriginalAudioContext = window.AudioContext;
    window.AudioContext = class extends OriginalAudioContext {
      constructor(...args) { super(...args); window.__audioContexts.push(this); }
      createGain(...args) { const gain = super.createGain(...args); window.__audioGains.push(gain); return gain; }
    };
    const originalStart = AudioBufferSourceNode.prototype.start;
    AudioBufferSourceNode.prototype.start = function(...args) { window.__audioStarts++; return originalStart.apply(this, args); };
  ` });
  await send('Page.navigate', { url: origin });
  try {
    await until(() => evaluate(`document.querySelector('#overlay') && !document.querySelector('#overlay').disabled`));
  } catch (error) {
    console.error('Browser startup errors:', errors);
    console.error('Page status:', await evaluate(`document.querySelector('#hint')?.textContent`));
    const capture = await send('Page.captureScreenshot', { format: 'png' });
    await writeFile(path.join(root, 'target/web-error.png'), Buffer.from(capture.data, 'base64'));
    throw error;
  }
  const click = async (x, y) => {
    await send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
    await send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });
    await delay(150);
  };
  const clickElement = async (selector) => {
    const point = await evaluate(`(() => { const r = document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
    await click(point.x, point.y);
  };
  const clickArena = async (x, y) => {
    const r = await evaluate(`(() => { const r = document.querySelector('canvas').getBoundingClientRect(); return {x:r.x,y:r.y,w:r.width,h:r.height}; })()`);
    await click(r.x + x * r.w / 1600, r.y + y * r.h / 900);
  };
  const screenshot = async (name) => {
    const image = await send('Page.captureScreenshot', { format: 'png' });
    await writeFile(path.join(root, 'target', name), Buffer.from(image.data, 'base64'));
    return image.data;
  };
  assert.equal(await evaluate(`getComputedStyle(document.querySelector('canvas')).filter`), 'grayscale(1)');
  assert.equal(await evaluate(`document.querySelector('#status').textContent`), 'Click in the game window to play');
  await until(() => evaluate(`document.fonts.check('20px "Proggy Clean"')`));
  await screenshot('web-ready.png');
  await clickArena(80, 600); // Any point in the game window activates it.
  assert.equal(await evaluate(`getComputedStyle(document.querySelector('canvas')).filter`), 'none');
  await until(() => evaluate(`window.__audioContexts.some(c => c.state === 'running') && window.__audioStarts > 0`));
  assert.equal(await evaluate(`window.__audioGains.length > 0 && window.__audioGains.every(node => node.gain.value === 0)`), true, 'Music starts muted');
  await clickArena(1260, 45);
  assert.equal(await evaluate(`window.__audioGains.some(node => node.gain.value > 0)`), true, 'Music can be unmuted');
  await clickArena(1260, 45);
  assert.equal(await evaluate(`window.__audioGains.every(node => node.gain.value === 0)`), true, 'Music can be muted again');
  assert.equal(await evaluate(`document.querySelector('#overlay').hidden`), true);
  assert.equal(await evaluate(`getComputedStyle(document.body).backgroundColor`), 'rgb(11, 15, 9)');
  assert.equal(await evaluate(`document.querySelector('main').firstElementChild.id`), 'stage');
  assert.equal(await evaluate(`document.querySelector('#stage').getBoundingClientRect().width <= 1360`), true);
  await until(() => evaluate(`document.querySelector('#patch-notes').textContent.length > 0`));
  assert.equal(await evaluate(`document.querySelector('#patch-notes').isContentEditable || !!document.querySelector('textarea, #export-notes')`), false);
  const publishedNotes = await evaluate(`document.querySelector('#patch-notes').textContent`);
  await clickElement('#theme-toggle');
  assert.equal(await evaluate(`document.documentElement.dataset.theme`), 'light');
  await evaluate(`localStorage.setItem('pyctf.notes-draft', 'Old draft must not replace published notes')`);
  await send('Page.navigate', { url: origin });
  await until(() => evaluate(`document.querySelector('#overlay') && !document.querySelector('#overlay').disabled`));
  assert.equal(await evaluate(`document.documentElement.dataset.theme`), 'light');
  await until(() => evaluate(`document.querySelector('#patch-notes').textContent.length > 0`));
  assert.equal(await evaluate(`document.querySelector('#patch-notes').textContent`), publishedNotes);
  await clickElement('#theme-toggle');
  await clickElement('#overlay');
  await send('Emulation.setDeviceMetricsOverride', { width: 1600, height: 1100, deviceScaleFactor: 1, mobile: false });
  await delay(150);
  await screenshot('web-lobby.png');
  await evaluate(`document.querySelector('#theme-toggle').click()`);
  await screenshot('web-light.png');
  await evaluate(`document.querySelector('#theme-toggle').click()`);
  await screenshot('web-dark.png');
  // Toggle NPC and Coin Race using scaled canvas coordinates, then start.
  await clickArena(1050, 367);
  await clickArena(800, 180);
  await clickArena(800, 730);
  await delay(500);
  if (process.argv.includes('--benchmark')) {
    await evaluate(`window.__recordFrames = true`);
    await delay(3000);
    const measurements = await evaluate(`(() => {
      window.__recordFrames = false;
      const summary = values => {
        values.sort((a, b) => a - b);
        return { count: values.length, mean: values.reduce((a, b) => a + b, 0) / values.length,
          p95: values[Math.floor(values.length * 0.95)], max: values.at(-1) };
      };
      return { cpuMs: summary(window.__frameCosts), intervalMs: summary(window.__frameIntervals) };
    })()`);
    await writeFile(path.join(root, 'target/web-perf.json'), JSON.stringify(measurements, null, 2));
    console.log('Browser sample (software GPU):', JSON.stringify(measurements));
  }
  await screenshot('web-coin-race.png');
  await evaluate(`window.dispatchEvent(new Event('blur'))`);
  assert.equal(await evaluate(`document.querySelector('#status').textContent`), 'Click in the window to resume');
  assert.equal(await evaluate(`getComputedStyle(document.querySelector('canvas')).filter`), 'grayscale(1)');
  await delay(100);
  const paused = await screenshot('web-paused.png');
  await delay(400);
  assert.equal(await screenshot('web-paused-check.png'), paused, 'Paused arena should remain unchanged');
  await clickArena(80, 600);
  assert.equal(await evaluate(`document.querySelector('#overlay').hidden`), true);
  await send('Emulation.setDeviceMetricsOverride', { width: 850, height: 750, deviceScaleFactor: 1, mobile: false });
  await delay(200);
  await screenshot('web-resized.png');
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await delay(100);
  await clickArena(800, 180); // Back to CTF.
  await clickArena(800, 730);
  await delay(200);
  await screenshot('web-ctf.png');
  assert.deepEqual(errors, [], 'No browser errors');
  console.log('Browser checks passed: WASM startup, user-gesture audio, scaled menu clicks, gameplay, focus pause/resume, resizing.');
  console.log('Screenshots: target/web-lobby.png, target/web-coin-race.png, target/web-paused.png, target/web-resized.png');
} finally {
  ws?.close();
  browser.kill();
}
