import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';

import {
  classifyStartupFailure,
  parseReadyPort,
  startBackend,
  stopBackend,
} from '../../electron/backend.mjs';

function fakeChild() {
  const child = new EventEmitter();
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.killCalls = 0;
  child.kill = () => {
    child.killCalls += 1;
    queueMicrotask(() => child.emit('close', 0, null));
    return true;
  };
  return child;
}

test('parseReadyPort accepts only a valid loopback readiness line', () => {
  assert.equal(parseReadyPort('[BACKEND] listening on 127.0.0.1:43210'), 43210);
  assert.equal(parseReadyPort('[BACKEND] listening on 0.0.0.0:43210'), null);
  assert.equal(parseReadyPort('[BACKEND] listening on 127.0.0.1:0'), null);
  assert.equal(parseReadyPort('backend output with 127.0.0.1:43210'), null);
});

test('classifyStartupFailure returns safe categories', () => {
  assert.equal(classifyStartupFailure({ reason: 'timeout' }).code, 'STARTUP_TIMEOUT');
  assert.equal(classifyStartupFailure({ reason: 'exit' }).code, 'BACKEND_EXITED');
  assert.equal(classifyStartupFailure({ reason: 'spawn' }).code, 'BACKEND_SPAWN_FAILED');
  const failure = classifyStartupFailure({ reason: 'spawn', detail: 'secret-token' });
  assert.doesNotMatch(failure.message, /secret-token/);
});

test('startBackend resolves loopback origin and enforces the environment contract', async () => {
  const child = fakeChild();
  let invocation;
  const resultPromise = startBackend({
    nodeBinary: 'C:/VelvLens/runtime/node.exe',
    backendEntry: 'C:/VelvLens/substore/backend/sub-store.bundle.js',
    frontendRoot: 'C:/VelvLens/substore/frontend',
    dataRoot: 'C:/Users/test/AppData/Roaming/VelvLens/sub-store-data',
    timeoutMs: 1000,
    spawnProcess: (binary, args, options) => {
      invocation = { binary, args, options };
      queueMicrotask(() => child.stdout.emit('data', Buffer.from('[BACKEND] listening on 127.0.0.1:49152\n')));
      return child;
    },
  });

  const result = await resultPromise;
  assert.equal(result.port, 49152);
  assert.equal(result.origin, 'http://127.0.0.1:49152');
  assert.equal(result.backendPath, '/velvlens-api');
  assert.equal(invocation.binary, 'C:/VelvLens/runtime/node.exe');
  assert.deepEqual(invocation.args, ['C:/VelvLens/substore/backend/sub-store.bundle.js']);
  assert.equal(invocation.options.windowsHide, true);
  assert.equal(invocation.options.env.SUB_STORE_BACKEND_API_HOST, '127.0.0.1');
  assert.equal(invocation.options.env.SUB_STORE_BACKEND_API_PORT, '0');
  assert.equal(invocation.options.env.SUB_STORE_BACKEND_MERGE, 'true');
  assert.equal(invocation.options.env.SUB_STORE_FRONTEND_BACKEND_PATH, '/velvlens-api');
  assert.equal(invocation.options.env.SUB_STORE_FRONTEND_PATH, 'C:/VelvLens/substore/frontend');
  assert.equal(invocation.options.env.SUB_STORE_DATA_BASE_PATH, 'C:/Users/test/AppData/Roaming/VelvLens/sub-store-data');
  await stopBackend(result.child);
});

test('startBackend reports timeout and pre-ready exit without child output', async () => {
  const timeoutChild = fakeChild();
  await assert.rejects(
    startBackend({
      nodeBinary: 'node.exe',
      backendEntry: 'bundle.js',
      frontendRoot: 'frontend',
      dataRoot: 'data',
      timeoutMs: 5,
      spawnProcess: () => timeoutChild,
    }),
    (error) => error.code === 'STARTUP_TIMEOUT' && !error.message.includes('secret')
  );

  const exitedChild = fakeChild();
  await assert.rejects(
    startBackend({
      nodeBinary: 'node.exe',
      backendEntry: 'bundle.js',
      frontendRoot: 'frontend',
      dataRoot: 'data',
      timeoutMs: 1000,
      spawnProcess: () => {
        queueMicrotask(() => exitedChild.emit('exit', 1, null));
        return exitedChild;
      },
    }),
    (error) => error.code === 'BACKEND_EXITED' && !error.message.includes('secret')
  );
});

test('stopBackend waits for bounded child shutdown', async () => {
  const child = fakeChild();
  await stopBackend(child, { timeoutMs: 100 });
  assert.equal(child.killCalls, 1);
});
