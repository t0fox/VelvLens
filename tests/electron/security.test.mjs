import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import { isAllowedLocalUrl, isExternalHttpUrl } from '../../electron/policy.mjs';

const mainSource = await readFile(new URL('../../electron/main.mjs', import.meta.url), 'utf8').catch(() => '');
const preloadSource = await readFile(new URL('../../electron/preload.mjs', import.meta.url), 'utf8').catch(() => '');
const errorPage = await readFile(new URL('../../electron/backend-error.html', import.meta.url), 'utf8').catch(() => '');

test('main process declares the Electron security boundary', () => {
  assert.match(mainSource, /contextIsolation:\s*true/u);
  assert.match(mainSource, /nodeIntegration:\s*false/u);
  assert.match(mainSource, /sandbox:\s*true/u);
  assert.match(mainSource, /setWindowOpenHandler/u);
  assert.match(mainSource, /will-navigate/u);
  assert.match(mainSource, /shell\.openExternal/u);
  assert.match(mainSource, /127\.0\.0\.1/u);
  assert.doesNotMatch(mainSource, /0\.0\.0\.0/u);
  assert.match(mainSource, /Content-Security-Policy/u);
});

test('preload exposes only the retry bridge', () => {
  assert.match(preloadSource, /contextBridge\.exposeInMainWorld\(['"]velvLens['"]/u);
  assert.match(preloadSource, /retryBackend/u);
  assert.doesNotMatch(preloadSource, /node:fs|node:child_process|shell\.openExternal/u);
  assert.doesNotMatch(preloadSource, /exposeInMainWorld\([^)]*,\s*\{[^}]*send/u);
});

test('error page contains the safe retry UI', () => {
  assert.match(errorPage, /Не удалось запустить локальный resolver/u);
  assert.match(errorPage, /Повторить запуск/u);
  assert.match(errorPage, /id="retry"/u);
});

test('URL policy accepts only the dynamic local backend origin', () => {
  const origin = 'http://127.0.0.1:49152';
  assert.equal(isAllowedLocalUrl(`${origin}/?magicpath=velvlens-api`, origin), true);
  assert.equal(isAllowedLocalUrl('http://127.0.0.1:49153/', origin), false);
  assert.equal(isAllowedLocalUrl('http://localhost:49152/', origin), false);
  assert.equal(isAllowedLocalUrl('file:///etc/passwd', origin), false);
  assert.equal(isExternalHttpUrl('https://example.com/docs'), true);
  assert.equal(isExternalHttpUrl('javascript:alert(1)'), false);
});
