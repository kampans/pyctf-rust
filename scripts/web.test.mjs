import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import { once } from 'node:events';
import { handler } from './serve-web.mjs';
import { fileURLToPath } from 'node:url';

test('static server serves WASM correctly, supports HEAD, and rejects missing files', async () => {
  const server = http.createServer(handler(fileURLToPath(new URL('../dist/', import.meta.url))));
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  try {
    const origin = `http://127.0.0.1:${server.address().port}`;
    const wasm = await fetch(`${origin}/pyctf.wasm`);
    assert.equal(wasm.status, 200);
    assert.equal(wasm.headers.get('content-type'), 'application/wasm');
    assert.deepEqual([...new Uint8Array(await wasm.arrayBuffer()).slice(0, 4)], [0, 97, 115, 109]);
    assert.equal((await fetch(origin)).status, 200);
    assert.equal((await fetch(`${origin}/missing.wasm`)).status, 404);
    assert.equal((await fetch(origin, { method: 'POST' })).status, 405);
    const head = await fetch(`${origin}/pyctf.wasm`, { method: 'HEAD' });
    assert.equal(head.status, 200);
    assert.equal((await head.arrayBuffer()).byteLength, 0);
    assert.equal((await fetch(`${origin}/..%5cCargo.toml`)).status, 403);
  } finally {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
});
