import { app, BrowserWindow, ipcMain, session, shell } from 'electron';
import { appendFile } from 'node:fs/promises';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  classifyStartupFailure,
  startBackend,
  stopBackend,
} from './backend.mjs';
import { isAllowedLocalUrl, isExternalHttpUrl } from './policy.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
export const LOCAL_CSP = "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self' https:; worker-src 'self' blob:; frame-src 'none'; object-src 'none'; base-uri 'self'; form-action 'self'";
const WINDOW_TITLE = 'VelvLens';
const ERROR_PAGE = join(__dirname, 'backend-error.html');

let mainWindow;
let backendRuntime;
let isQuitting = false;
let lastFailureCode;

export function resolveRuntimePaths({ packaged = app.isPackaged, resourcesPath = process.resourcesPath, projectRoot = resolve(__dirname, '..') } = {}) {
  const root = packaged ? resourcesPath : join(projectRoot, '.build');
  return {
    nodeBinary: join(root, 'runtime', 'node.exe'),
    backendEntry: join(root, 'substore', 'backend', 'sub-store.bundle.js'),
    frontendRoot: join(root, 'substore', 'frontend'),
    dataRoot: join(app.getPath('userData'), 'sub-store-data'),
  };
}

function safeFailureCode(code) {
  return new Set(['STARTUP_TIMEOUT', 'BACKEND_EXITED', 'BACKEND_SPAWN_FAILED', 'BACKEND_START_FAILED']).has(code)
    ? code
    : 'BACKEND_START_FAILED';
}

function safeFailure(error) {
  const code = safeFailureCode(error?.code);
  const failure = classifyStartupFailure({ code });
  return { code: failure.code, message: failure.message };
}

function configureHeaders() {
  session.defaultSession.webRequest.onHeadersReceived(
    { urls: ['http://127.0.0.1:*/*', 'file://*/*'] },
    (details, callback) => {
      callback({
        responseHeaders: {
          ...details.responseHeaders,
          'Content-Security-Policy': [LOCAL_CSP],
        },
      });
    }
  );
}

function configureNavigation(win) {
  win.webContents.setWindowOpenHandler(({ url }) => {
    if (isExternalHttpUrl(url)) void shell.openExternal(url);
    return { action: 'deny' };
  });

  win.webContents.on('will-navigate', (event, url) => {
    const errorPageUrl = pathToFileURL(ERROR_PAGE).href;
    if (url.startsWith(errorPageUrl)) return;
    if (backendRuntime && isAllowedLocalUrl(url, backendRuntime.origin)) return;
    event.preventDefault();
    if (isExternalHttpUrl(url)) void shell.openExternal(url);
  });

  win.webContents.on('page-title-updated', (event) => {
    event.preventDefault();
    win.setTitle(WINDOW_TITLE);
  });
}

export async function showBackendError(win, failure) {
  const code = safeFailureCode(failure?.code);
  await win.loadFile(ERROR_PAGE, { query: { code } });
}

export async function loadSubStore(win, paths = resolveRuntimePaths()) {
  if (backendRuntime) {
    const previous = backendRuntime.child;
    backendRuntime = undefined;
    await stopBackend(previous);
  }

  const started = await startBackend(paths);
  backendRuntime = started;
  const url = `${started.origin}/?magicpath=${started.backendPath.slice(1)}`;
  try {
    await win.loadURL(url);
    win.setTitle(WINDOW_TITLE);
    return started;
  } catch {
    backendRuntime = undefined;
    await stopBackend(started.child);
    throw Object.assign(new Error('Unable to load local resolver'), { code: 'BACKEND_START_FAILED' });
  }
}

export async function createMainWindow() {
  if (mainWindow && !mainWindow.isDestroyed()) return mainWindow;
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 650,
    title: WINDOW_TITLE,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      preload: join(__dirname, 'preload.mjs'),
    },
  });
  configureNavigation(mainWindow);
  mainWindow.on('closed', () => {
    const child = backendRuntime?.child;
    backendRuntime = undefined;
    void stopBackend(child);
    mainWindow = undefined;
  });

  try {
    await loadSubStore(mainWindow);
    lastFailureCode = undefined;
  } catch (error) {
    const failure = safeFailure(error);
    lastFailureCode = failure.code;
    await showBackendError(mainWindow, failure);
  }
  return mainWindow;
}

async function retryBackend(event) {
  if (!mainWindow || event.sender !== mainWindow.webContents) {
    return { ok: false, code: 'BACKEND_START_FAILED' };
  }
  try {
    await loadSubStore(mainWindow);
    return { ok: true };
  } catch (error) {
    const failure = safeFailure(error);
    await showBackendError(mainWindow, failure);
    return { ok: false, code: failure.code };
  }
}

async function shutdown() {
  const child = backendRuntime?.child;
  backendRuntime = undefined;
  await stopBackend(child);
}

async function emitSmokeMarker(message) {
  if (process.env.VELVLENS_SMOKE !== '1') return;
  const line = `[VELVLENS_SMOKE] ${message}`;
  process.stdout.write(`${line}\n`);
  if (process.env.VELVLENS_SMOKE_FILE) {
    try {
      await appendFile(process.env.VELVLENS_SMOKE_FILE, `${line}\n`, 'utf8');
    } catch {
      // Smoke signaling must never affect the application lifecycle.
    }
  }
}

const gotLock = app.requestSingleInstanceLock();
if (!gotLock) {
  app.quit();
} else {
  app.on('second-instance', () => mainWindow?.focus());
  app.whenReady().then(async () => {
    configureHeaders();
    ipcMain.handle('backend:retry', retryBackend);
    await createMainWindow();
    if (backendRuntime) {
      await emitSmokeMarker(`READY title=${WINDOW_TITLE} backend=loopback child=${backendRuntime.child.pid ?? 0}`);
    } else {
      await emitSmokeMarker(`ERROR code=${lastFailureCode ?? 'BACKEND_START_FAILED'}`);
    }
    if (process.env.VELVLENS_SMOKE_EXIT === '1') setTimeout(() => app.quit(), 400);
    app.on('activate', () => void createMainWindow());
  });
  app.on('before-quit', (event) => {
    if (isQuitting) return;
    event.preventDefault();
    isQuitting = true;
    void shutdown().finally(() => app.quit());
  });
  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
  });
}
