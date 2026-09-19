import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { access, readdir, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { listPackage } from '@electron/asar';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const packageRoot = resolve(process.argv[2] ?? join(root, 'release', 'win-unpacked'));
const resourcesRoot = join(packageRoot, 'resources');
const EXPECTED = {
  backend: '8c2a695663b29a339f27651c6c683d54b04c7952',
  frontend: '628403546cca2c00056cad35d5e285f9c9874b6d',
  node: '3331e1ffe19874215472217c5e94f5a0c6d8e18c4ac7111d3937aa0ad5e9b4a5',
};

async function requireFile(filePath) {
  await access(filePath);
  return filePath;
}

async function sha256(filePath) {
  const hash = createHash('sha256');
  hash.update(await readFile(filePath));
  return hash.digest('hex');
}

async function allFiles(directory, prefix = '') {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = join(prefix, entry.name);
    if (entry.isDirectory()) result.push(...await allFiles(join(directory, entry.name), relative));
    else result.push(relative);
  }
  return result;
}

const manifestPath = await requireFile(join(resourcesRoot, 'substore', 'manifest.json'));
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
assert.equal(manifest.backend.sha, EXPECTED.backend, 'backend pin');
assert.equal(manifest.frontend.sha, EXPECTED.frontend, 'frontend pin');
assert.equal(manifest.node.version, '24.15.0', 'Node runtime version');

const nodePath = await requireFile(join(resourcesRoot, 'runtime', 'node.exe'));
assert.equal(await sha256(nodePath), EXPECTED.node, 'Node runtime SHA-256');
await requireFile(join(resourcesRoot, 'substore', 'backend', 'sub-store.bundle.js'));
await requireFile(join(resourcesRoot, 'substore', 'backend', 'package.json'));
await requireFile(join(resourcesRoot, 'substore', 'frontend', 'index.html'));
await requireFile(join(resourcesRoot, 'licenses', 'SUB-STORE-AGPL-3.0.txt'));
await requireFile(join(resourcesRoot, 'licenses', 'SUB-STORE-FRONTEND-GPL-3.0.txt'));
await requireFile(join(resourcesRoot, 'licenses', 'NODE-24.15.0.txt'));
await requireFile(join(resourcesRoot, 'THIRD_PARTY_NOTICES.md'));

const resourceFiles = await allFiles(resourcesRoot);
assert.equal(resourceFiles.some((file) => /node_modules|src[\\/]resolver|sublens\.exe|0\.0\.0\.0/iu.test(file)), false, 'forbidden package resource');

const asarPath = await requireFile(join(resourcesRoot, 'app.asar'));
const asarFiles = listPackage(asarPath).map((file) => file.replaceAll('\\', '/').replace(/^\/+/, ''));
assert.equal(asarFiles.some((file) => /node_modules|src[\\/]resolver|sublens\.exe|Cargo\.toml|Cargo\.lock/iu.test(file)), false, 'forbidden app.asar entry');
assert.equal(asarFiles.includes('electron/main.mjs'), true, 'main process is packaged');
assert.equal(asarFiles.includes('electron/preload.mjs'), true, 'preload is packaged');

console.log(`Package verification passed: ${packageRoot}`);
