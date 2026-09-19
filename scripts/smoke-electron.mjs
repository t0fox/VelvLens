import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { access, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const electronBinary = join(root, 'node_modules', 'electron', 'dist', 'electron.exe');
await access(electronBinary);
const userDataRoot = await mkdtemp(join(tmpdir(), 'velvlens-electron-smoke-'));
const child = spawn(electronBinary, ['.', `--user-data-dir=${userDataRoot}`, '--disable-gpu'], {
  cwd: root,
  env: { ...process.env, VELVLENS_SMOKE: '1', VELVLENS_SMOKE_EXIT: '1' },
  stdio: ['ignore', 'pipe', 'pipe'],
  windowsHide: true,
});

let output = '';
let childPid = 0;
const ready = new Promise((resolveReady, rejectReady) => {
  const timeout = setTimeout(() => rejectReady(new Error('Electron smoke timed out waiting for BrowserWindow readiness')), 45_000);
  const onData = (chunk) => {
    output += Buffer.from(chunk).toString('utf8');
    const lines = output.split(/\r?\n/u);
    output = lines.pop() ?? '';
    for (const line of lines) {
      const match = /^\[VELVLENS_SMOKE\] READY title=VelvLens backend=loopback child=(\d+)$/u.exec(line.trim());
      if (match) {
        clearTimeout(timeout);
        childPid = Number(match[1]);
        resolveReady();
        return;
      }
      const errorMatch = /^\[VELVLENS_SMOKE\] ERROR code=([A-Z_]+)$/u.exec(line.trim());
      if (errorMatch) {
        clearTimeout(timeout);
        rejectReady(new Error(`Electron smoke reported ${errorMatch[1]}`));
        return;
      }
    }
  };
  child.stdout.on('data', onData);
  child.stderr.on('data', () => {});
  child.once('error', rejectReady);
});

try {
  await ready;
  const exitCode = await new Promise((resolveExit, rejectExit) => {
    child.once('error', rejectExit);
    child.once('exit', (code) => resolveExit(code));
  });
  assert.equal(exitCode, 0, 'Electron smoke exit code');
  if (childPid > 0) {
    assert.throws(() => process.kill(childPid, 0), 'Sub-Store backend child still exists after Electron exit');
  }
  console.log('Electron smoke passed: BrowserWindow, local backend lifecycle, and shutdown');
} finally {
  if (!child.killed && child.exitCode === null) {
    child.kill();
    await new Promise((resolveExit) => {
      const timeout = setTimeout(resolveExit, 5_000);
      child.once('exit', () => {
        clearTimeout(timeout);
        resolveExit();
      });
    });
  }
  await rm(userDataRoot, { recursive: true, force: true, maxRetries: 8, retryDelay: 250 });
}
