import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { mkdtemp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import {
  assertPinnedRepo,
  validateStaging,
} from '../../scripts/build-substore.mjs';

const backendSha = '8c2a695663b29a339f27651c6c683d54b04c7952';
const frontendSha = '628403546cca2c00056cad35d5e285f9c9874b6d';

test('assertPinnedRepo accepts the exact upstream gitlinks', async () => {
  await assert.doesNotReject(assertPinnedRepo('third_party/sub-store', backendSha));
  await assert.doesNotReject(assertPinnedRepo('third_party/sub-store-frontend', frontendSha));
});

test('assertPinnedRepo rejects a wrong gitlink', async () => {
  await assert.rejects(
    assertPinnedRepo('third_party/sub-store', '0'.repeat(40)),
    /expected pin/
  );
});

test('validateStaging requires generated files and rejects node_modules', async () => {
  const root = await mkdtemp(join(tmpdir(), 'velvlens-substore-stage-'));
  await mkdir(join(root, 'backend'), { recursive: true });
  await mkdir(join(root, 'frontend', 'assets'), { recursive: true });
  await writeFile(join(root, 'backend', 'sub-store.bundle.js'), 'bundle');
  await writeFile(join(root, 'backend', 'runtime-manifest.json'), '{}');
  await writeFile(join(root, 'backend', 'package.json'), '{"type":"commonjs"}');
  await writeFile(join(root, 'frontend', 'index.html'), '<!doctype html>');
  await writeFile(join(root, 'frontend', 'assets', 'app.js'), 'chunk');
  await writeFile(join(root, 'manifest.json'), '{}');

  await assert.doesNotReject(validateStaging(root));
  await mkdir(join(root, 'node_modules', 'bad-package'), { recursive: true });
  await assert.rejects(validateStaging(root), /node_modules/);
});
