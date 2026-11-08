import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = fileURLToPath(new URL('../', import.meta.url));
const result = spawnSync('cargo', ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown'], {
  cwd: root, stdio: 'inherit', windowsHide: true,
});
if (result.error || result.status !== 0) {
  console.error(result.error?.message ?? 'Web build failed. First run: rustup target add wasm32-unknown-unknown');
  process.exit(1);
}
const dist = path.join(root, 'dist');
await mkdir(path.join(dist, 'assets'), { recursive: true });
await mkdir(path.join(dist, 'vendor'), { recursive: true });
for (const name of ['index.html', 'styles.css', 'app.js', 'runtime-compat.js', 'theme.js', 'updates.txt']) {
  await copyFile(path.join(root, 'web', name), path.join(dist, name));
}
for (const name of ['mq_js_bundle.js', 'LICENSE-MIT', 'LICENSE-APACHE', 'README.md', 'ProggyClean.ttf', 'LICENSE-ProggyClean.txt']) {
  await copyFile(path.join(root, 'web/vendor', name), path.join(dist, 'vendor', name));
}
await copyFile(path.join(root, 'target/wasm32-unknown-unknown/release/pyctf.wasm'), path.join(dist, 'pyctf.wasm'));
for (const name of ['background.png', 'red_flag_refined_45x45.png', 'blue_flag_refined_45x45.png', 'player_red.png', 'player_blue.png']) {
  await copyFile(path.join(root, 'assets', name), path.join(dist, 'assets', name));
}
// The coin texture is loaded from disk; sound files are embedded in the WASM.
await copyFile(path.join(root, 'assets/coin.png'), path.join(dist, 'assets/coin.png'));
const wasm = await readFile(path.join(dist, 'pyctf.wasm'));
console.log(`Web build ready: dist/ (${(wasm.length / 1024 / 1024).toFixed(2)} MiB WASM)`);
