import http from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../dist/', import.meta.url));
const mime = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.wasm': 'application/wasm', '.png': 'image/png', '.ttf': 'font/ttf', '.md': 'text/plain; charset=utf-8' };

export function handler(directory) {
  return async (req, res) => {
    if (req.method !== 'GET' && req.method !== 'HEAD') {
      res.writeHead(405, { Allow: 'GET, HEAD' }).end();
      return;
    }
    try {
      let pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
      if (pathname.split(/[\\/]/).includes('..')) {
        res.writeHead(403).end('Forbidden');
        return;
      }
      if (pathname.endsWith('/')) pathname += 'index.html';
      const filename = path.resolve(directory, `.${pathname}`);
      const relative = path.relative(directory, filename);
      if (relative.startsWith('..') || path.isAbsolute(relative)) {
        res.writeHead(403).end('Forbidden');
        return;
      }
      const bytes = await readFile(filename);
      res.writeHead(200, {
        'Content-Type': mime[path.extname(filename)] ?? 'application/octet-stream',
        'Content-Length': bytes.length,
        'Cache-Control': 'no-store',
        'X-Content-Type-Options': 'nosniff',
      });
      res.end(req.method === 'HEAD' ? undefined : bytes);
    } catch {
      res.writeHead(404, { 'Content-Type': 'text/plain' }).end('Not found');
    }
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await stat(path.join(root, 'pyctf.wasm')); }
  catch { console.error('Build the game first: npm run web:build'); process.exit(1); }
  const at = process.argv.indexOf('--port');
  const port = Number(at >= 0 ? process.argv[at + 1] : 8080);
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid --port');
  const server = http.createServer(handler(root));
  server.on('error', (error) => { console.error(error.message); process.exitCode = 1; });
  server.listen(port, '127.0.0.1', () => console.log(`pyCTF: http://127.0.0.1:${port} (Ctrl+C to stop)`));
}
