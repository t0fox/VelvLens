import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { startBackend, stopBackend } from '../electron/backend.mjs';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const fixtureDirectory = join(root, 'tests', 'fixtures');
const fixtures = [
  ['fixture-vless', 'substore-vless.txt'],
  ['fixture-base64', 'substore-base64.txt'],
  ['fixture-hysteria2', 'substore-hysteria2.txt'],
  ['fixture-json', 'substore-json.json'],
  ['fixture-mixed', 'substore-mixed.txt'],
];

function assertStatus(response, expected, step) {
  assert.equal(response.status, expected, `${step} status`);
}

function containsName(value, name) {
  if (Array.isArray(value)) return value.some((item) => containsName(item, name));
  if (!value || typeof value !== 'object') return false;
  if (value.name === name) return true;
  return Object.values(value).some((item) => containsName(item, name));
}

async function json(response, step) {
  const value = await response.json();
  assert.equal(value.status, 'success', `${step} status field`);
  return value;
}

const dataRoot = await mkdtemp(join(tmpdir(), 'velvlens-substore-smoke-'));
const runtime = await startBackend({
  nodeBinary: join(root, '.build', 'runtime', 'node.exe'),
  backendEntry: join(root, '.build', 'substore', 'backend', 'sub-store.bundle.js'),
  frontendRoot: join(root, '.build', 'substore', 'frontend'),
  dataRoot,
});

try {
  const rootResponse = await fetch(`${runtime.origin}/`);
  assertStatus(rootResponse, 200, 'merged frontend');
  assert.match(await rootResponse.text(), /<html/iu);

  const healthResponse = await fetch(`${runtime.origin}${runtime.backendPath}/api/utils/env`);
  assertStatus(healthResponse, 200, 'magic-path health');
  await json(healthResponse, 'magic-path health');

  const magicPathResponse = await fetch(`${runtime.origin}/?magicpath=${runtime.backendPath.slice(1)}`);
  assertStatus(magicPathResponse, 200, 'magic-path frontend');
  assert.match(await magicPathResponse.text(), /<html/iu);

  for (const [name, fileName] of fixtures) {
    const content = (await readFile(join(fixtureDirectory, fileName), 'utf8')).trim();
    const createResponse = await fetch(`${runtime.origin}${runtime.backendPath}/api/subs`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ name, displayName: name, source: 'local', content, process: [] }),
    });
    assert.ok([200, 201].includes(createResponse.status), `${name} create status ${createResponse.status}`);
    await json(createResponse, `${name} create`);
  }

  const listResponse = await fetch(`${runtime.origin}${runtime.backendPath}/api/subs`);
  assertStatus(listResponse, 200, 'subscription list');
  const subscriptions = await json(listResponse, 'subscription list');
  for (const [name] of fixtures) {
    assert.equal(containsName(subscriptions.data, name), true, `${name} appears in subscription list`);
  }

  const exportResponse = await fetch(`${runtime.origin}${runtime.backendPath}/download/fixture-vless/V2Ray`);
  assertStatus(exportResponse, 200, 'V2Ray export');
  assert.match(exportResponse.headers.get('content-type') ?? '', /text\/plain/iu);
  const encoded = (await exportResponse.text()).trim();
  assert.ok(encoded.length > 0, 'V2Ray export is non-empty');
  const decoded = Buffer.from(encoded, 'base64').toString('utf8');
  assert.match(decoded, /vless:\/\//iu);
  assert.match(decoded, /example\.com/iu);

  console.log('Sub-Store smoke passed: frontend, health, VLESS/Base64/Hysteria2/JSON/mixed fixtures, and V2Ray export');
} finally {
  await stopBackend(runtime.child);
  await rm(dataRoot, { recursive: true, force: true });
}
