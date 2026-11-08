import { readFile } from 'node:fs/promises';

const dist = new URL('../dist/', import.meta.url);
const html = await readFile(new URL('index.html', dist), 'utf8');
for (const match of html.matchAll(/(?:src|href)="([^"#]+)"/g)) {
  if (!match[1].includes('://')) await readFile(new URL(match[1], dist));
}
const module = new WebAssembly.Module(await readFile(new URL('pyctf.wasm', dist)));
const exports = new Set(WebAssembly.Module.exports(module).map((entry) => entry.name));
for (const name of ['main', 'memory', 'pyctf_ready', 'pyctf_set_active']) {
  if (!exports.has(name)) throw new Error(`Missing WASM export: ${name}`);
}
for (const entry of WebAssembly.Module.imports(module)) {
  if (entry.module !== 'env') throw new Error(`Unexpected import module: ${entry.module}`);
  if (entry.name.includes('wbindgen')) throw new Error('wasm-bindgen needs a different loader');
}
for (const name of ['background.png', 'red_flag_refined_45x45.png', 'blue_flag_refined_45x45.png', 'player_red.png', 'player_blue.png', 'coin.png']) {
  await readFile(new URL(`assets/${name}`, dist));
}
console.log('Web package valid: HTML references, WASM exports/imports, and game assets.');
