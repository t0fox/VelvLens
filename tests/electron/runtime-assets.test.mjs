import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { mkdtemp, readFile, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { downloadVerified, runtimePath } from '../../scripts/fetch-node-runtime.mjs';

const payload = Buffer.from('velvlens-runtime-test');
const payloadHash = createHash('sha256').update(payload).digest('hex');

function servePayload() {
  const server = createServer((request, response) => {
    if (request.url === '/node.exe') {
      response.writeHead(200, { 'content-type': 'application/octet-stream' });
      response.end(payload);
      return;
    }
    response.writeHead(404);
    response.end();
  });

  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      resolve({ server, url: `http://127.0.0.1:${port}/node.exe` });
    });
  });
}

test('downloadVerified writes a matching response atomically', async (t) => {
  const { server, url } = await servePayload();
  t.after(() => server.close());
  const root = await mkdtemp(join(tmpdir(), 'velvlens-runtime-test-'));
  const destination = join(root, 'node.exe');

  await downloadVerified({ url, destination, sha256: payloadHash });

  assert.deepEqual(await readFile(destination), payload);
  await assert.rejects(stat(`${destination}.part`));
});

test('downloadVerified rejects a mismatching hash without a final file', async (t) => {
  const { server, url } = await servePayload();
  t.after(() => server.close());
  const root = await mkdtemp(join(tmpdir(), 'velvlens-runtime-test-'));
  const destination = join(root, 'node.exe');

  await assert.rejects(
    downloadVerified({ url, destination, sha256: '0'.repeat(64) }),
    /SHA-256 mismatch/
  );
  await assert.rejects(stat(destination));
  await assert.rejects(stat(`${destination}.part`));
});

test('runtimePath resolves packaged and development locations', () => {
  assert.equal(
    runtimePath({ packaged: true, resourcesPath: 'C:/Program Files/VelvLens/resources', projectRoot: 'ignored' }),
    join('C:/Program Files/VelvLens/resources', 'runtime', 'node.exe')
  );
  assert.equal(
    runtimePath({ packaged: false, resourcesPath: 'ignored', projectRoot: 'C:/work/VelvLens' }),
    join('C:/work/VelvLens', '.build', 'runtime', 'node.exe')
  );
});
