import assert from 'node:assert/strict';
import { access, copyFile, mkdtemp, readFile, rm } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const source = join(root, 'release', 'VelvLens-portable.exe');
await access(source);

const cleanRoot = await mkdtemp(join(tmpdir(), 'velvlens-clean-install-'));
const portable = join(cleanRoot, 'VelvLens-portable.exe');
const marker = join(cleanRoot, 'smoke-marker.txt');
await copyFile(source, portable);

const child = spawn(portable, ['--disable-gpu'], {
  cwd: cleanRoot,
  env: { ...process.env, VELVLENS_SMOKE: '1', VELVLENS_SMOKE_EXIT: '1', VELVLENS_SMOKE_FILE: marker },
  stdio: ['ignore', 'pipe', 'pipe'],
  windowsHide: true,
});

let backendPid = 0;
const ready = new Promise((resolveReady, rejectReady) => {
  const deadline = Date.now() + 60_000;
  const check = async () => {
    try {
      const markerText = await readFile(marker, 'utf8');
      const readyMatch = /^\[VELVLENS_SMOKE\] READY title=VelvLens backend=loopback child=(\d+)$/mu.exec(markerText);
      if (readyMatch) {
        backendPid = Number(readyMatch[1]);
        resolveReady();
        return;
      }
      const errorMatch = /^\[VELVLENS_SMOKE\] ERROR code=([A-Z_]+)$/mu.exec(markerText);
      if (errorMatch) {
        rejectReady(new Error(`portable EXE reported ${errorMatch[1]}`));
        return;
      }
    } catch {
      // The marker is created by the packaged app after the BrowserWindow is ready.
    }
    if (Date.now() >= deadline) {
      rejectReady(new Error('portable EXE timed out'));
      return;
    }
    setTimeout(() => void check(), 250);
  };
  child.once('error', rejectReady);
  void check();
});

try {
  await ready;
  const exitCode = await new Promise((resolveExit, rejectExit) => {
    child.once('error', rejectExit);
    child.once('exit', (code) => resolveExit(code));
  });
  assert.equal(exitCode, 0, 'portable EXE exit code');
  if (backendPid > 0) assert.throws(() => process.kill(backendPid, 0), 'portable backend still exists after exit');
  console.log('Clean-install smoke passed: copied portable EXE launched from a fresh directory and shut down cleanly');
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
  await rm(cleanRoot, { recursive: true, force: true, maxRetries: 8, retryDelay: 250 });
}
