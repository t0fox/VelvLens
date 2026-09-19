import { spawn } from 'node:child_process';
import { mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';

export const BACKEND_PATH = '/velvlens-api';
const DEFAULT_START_TIMEOUT_MS = 30_000;
const DEFAULT_STOP_TIMEOUT_MS = 5_000;

export function parseReadyPort(line) {
  const match = /(?:^|\s)\[BACKEND\]\s+listening\s+on\s+127\.0\.0\.1:(\d{1,5})\s*$/u.exec(String(line).trim());
  if (!match) return null;
  const port = Number(match[1]);
  return port >= 1 && port <= 65_535 ? port : null;
}

export function classifyStartupFailure({ reason, code } = {}) {
  if (reason === 'timeout' || code === 'STARTUP_TIMEOUT') {
    return { code: 'STARTUP_TIMEOUT', message: 'Локальный resolver не сообщил о готовности вовремя.' };
  }
  if (reason === 'exit' || code === 'BACKEND_EXITED') {
    return { code: 'BACKEND_EXITED', message: 'Локальный resolver завершился до готовности.' };
  }
  if (reason === 'spawn' || code === 'BACKEND_SPAWN_FAILED') {
    return { code: 'BACKEND_SPAWN_FAILED', message: 'Не удалось запустить локальный resolver.' };
  }
  return { code: 'BACKEND_START_FAILED', message: 'Не удалось запустить локальный resolver.' };
}

function safeError(failure) {
  const error = new Error(failure.message);
  error.code = failure.code;
  return error;
}

export async function startBackend({
  nodeBinary,
  backendEntry,
  frontendRoot,
  dataRoot,
  timeoutMs = DEFAULT_START_TIMEOUT_MS,
  spawnProcess = spawn,
} = {}) {
  try {
    await mkdir(dataRoot, { recursive: true });
  } catch {
    return Promise.reject(safeError(classifyStartupFailure({ reason: 'spawn' })));
  }

  const environment = {
    ...process.env,
    SUB_STORE_BACKEND_API_HOST: '127.0.0.1',
    SUB_STORE_BACKEND_API_PORT: '0',
    SUB_STORE_BACKEND_MERGE: 'true',
    SUB_STORE_FRONTEND_BACKEND_PATH: BACKEND_PATH,
    SUB_STORE_FRONTEND_PATH: frontendRoot,
    SUB_STORE_DATA_BASE_PATH: dataRoot,
    SUB_STORE_BACKEND_CUSTOM_NAME: 'VelvLens',
  };

  let child;
  try {
    child = spawnProcess(nodeBinary, [backendEntry], {
      cwd: dirname(backendEntry),
      env: environment,
      stdio: ['ignore', 'pipe', 'pipe'],
      windowsHide: true,
    });
  } catch {
    return Promise.reject(safeError(classifyStartupFailure({ reason: 'spawn' })));
  }

  return new Promise((resolve, reject) => {
    let settled = false;
    let lineBuffer = '';
    let timer;

    const cleanup = () => {
      clearTimeout(timer);
      child.stdout?.removeListener('data', onOutput);
      child.stderr?.removeListener('data', onOutput);
      child.removeListener('error', onError);
      child.removeListener('exit', onExit);
    };

    const fail = async (failure) => {
      if (settled) return;
      settled = true;
      cleanup();
      try {
        child.kill();
      } catch {
        // The process may have exited between the readiness failure and cleanup.
      }
      reject(safeError(failure));
    };

    const onOutput = (chunk) => {
      lineBuffer += Buffer.from(chunk).toString('utf8');
      const lines = lineBuffer.split(/\r?\n/u);
      lineBuffer = lines.pop() ?? '';
      for (const line of lines) {
        const port = parseReadyPort(line);
        if (port === null || settled) continue;
        settled = true;
        cleanup();
        resolve({ child, port, origin: `http://127.0.0.1:${port}`, backendPath: BACKEND_PATH });
        return;
      }
      if (lineBuffer.length > 4096) lineBuffer = lineBuffer.slice(-4096);
    };

    const onError = () => void fail(classifyStartupFailure({ reason: 'spawn' }));
    const onExit = () => void fail(classifyStartupFailure({ reason: 'exit' }));

    child.stdout?.on('data', onOutput);
    child.stderr?.on('data', onOutput);
    child.once('error', onError);
    child.once('exit', onExit);
    timer = setTimeout(() => void fail(classifyStartupFailure({ reason: 'timeout' })), timeoutMs);
  });
}

export function stopBackend(child, { timeoutMs = DEFAULT_STOP_TIMEOUT_MS } = {}) {
  if (!child || child.exitCode !== null && child.exitCode !== undefined || child.signalCode) {
    return Promise.resolve();
  }

  return new Promise((resolve) => {
    let settled = false;
    const finish = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      child.removeListener('close', finish);
      child.removeListener('exit', finish);
      resolve();
    };
    const timer = setTimeout(finish, timeoutMs);
    child.once('close', finish);
    child.once('exit', finish);
    try {
      child.kill();
    } catch {
      finish();
    }
  });
}
