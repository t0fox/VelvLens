import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { mkdir, rename, rm } from 'node:fs/promises';
import { basename, dirname, join } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

const NODE_VERSION = '24.15.0';
const NODE_SHA256 = '3331e1ffe19874215472217c5e94f5a0c6d8e18c4ac7111d3937aa0ad5e9b4a5';
const NODE_URL = `https://nodejs.org/dist/v${NODE_VERSION}/win-x64/node.exe`;

export function runtimePath({ packaged, resourcesPath, projectRoot }) {
  if (packaged) return join(resourcesPath, 'runtime', 'node.exe');
  return join(projectRoot, '.build', 'runtime', 'node.exe');
}

async function sha256File(filePath) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(filePath)) hash.update(chunk);
  return hash.digest('hex');
}

export async function downloadVerified({ url, destination, sha256, fetchImpl = fetch }) {
  if (!/^[a-f0-9]{64}$/i.test(sha256)) throw new Error('Invalid SHA-256 value');
  await mkdir(dirname(destination), { recursive: true });
  const partPath = `${destination}.part`;
  await rm(partPath, { force: true });

  try {
    const response = await fetchImpl(url, { redirect: 'follow' });
    if (!response.ok || !response.body) {
      throw new Error(`Runtime download failed with HTTP ${response.status}`);
    }

    await pipeline(Readable.fromWeb(response.body), createWriteStream(partPath));

    const actual = await sha256File(partPath);
    if (actual.toLowerCase() !== sha256.toLowerCase()) {
      throw new Error(`SHA-256 mismatch for ${basename(destination)}`);
    }
    await rename(partPath, destination);
  } catch (error) {
    await rm(partPath, { force: true });
    throw error;
  }
}

async function main() {
  const projectRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
  const destination = join(projectRoot, '.build', 'runtime', 'node.exe');
  await downloadVerified({ url: NODE_URL, destination, sha256: NODE_SHA256 });
  console.log(`Verified Node.js ${NODE_VERSION} runtime at ${destination}`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  await main();
}

export { NODE_SHA256, NODE_URL, NODE_VERSION };
