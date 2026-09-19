# VelvLens Sub-Store Electron Design

**Date:** 2026-09-19  
**Status:** Validated design for implementation  
**Branch:** `codex/velvlens-substore-electron`

## Goal

Replace VelvLens's custom Rust/egui subscription resolver with a small Windows Electron application that starts a pinned, unmodified Sub-Store backend locally, serves the pinned official Sub-Store frontend from the same localhost origin, and packages every runtime dependency required by the user-facing EXE.

## Evidence and pinned upstream

The current VelvLens repository is a Rust 2021 application named `sublens`, with the resolver, protocol parsers, share URI serializers, export pipeline, and egui UI owned by `src/` and covered by Rust tests. The migration branch starts at `f183725aa95b1a0c29188247dfd427265cd5cb66` and does not preserve a second application after migration.

The upstream sources are fixed as git submodules:

| Component | Repository | Commit | Upstream build |
| --- | --- | --- | --- |
| Backend | `https://github.com/sub-store-org/Sub-Store.git` | `8c2a695663b29a339f27651c6c683d54b04c7952` | `pnpm install --frozen-lockfile` then `pnpm bundle:esbuild` |
| Frontend | `https://github.com/sub-store-org/Sub-Store-Front-End.git` | `628403546cca2c00056cad35d5e285f9c9874b6d` | `pnpm install --frozen-lockfile` then `pnpm build` |

The verified upstream backend is version `2.39.9`, the frontend is version `2.31.2`, and both upstream `.node-version` files specify Node `24.15.0`. The backend bundle produces `dist/sub-store.bundle.js` and a runtime manifest with no external npm packages, while retaining Node builtins and the documented optional `shoutrrr` external binary. The bundle was run successfully without `node_modules` in a clean staging directory.

Sub-Store is AGPL-3.0. The frontend is GPL-3.0. VelvLens's wrapper license remains separate, but the distribution is explicitly composite and is never described as MIT-only.

## Architecture

The repository has one product owner after the migration:

```text
VelvLens.exe
  Electron main process
    ├─ starts bundled Node 24.15.0
    │    └─ Sub-Store dist/sub-store.bundle.js
    │         ├─ 127.0.0.1:dynamic-port
    │         └─ merged static frontend from Sub-Store dist/
    └─ BrowserWindow
         └─ http://127.0.0.1:<port>/?magicpath=velvlens-api
```

The app does not reimplement parsing, protocol detection, subscription fetching, JSON conversion, URI serialization, QR generation, or export. The only custom logic is process lifecycle, dynamic-port discovery, error recovery, Electron security policy, packaging, and build verification.

### Upstream integration

The two official repositories are added as submodules under `third_party/`. The build script refuses to build when either submodule is absent or at a different commit. It runs the upstream package-manager/build commands without modifying upstream source files, then copies only the generated frontend `dist/` and backend `dist/sub-store.bundle.js` into a generated staging directory consumed by Electron Builder.

The generated runtime contains:

- `substore/backend/sub-store.bundle.js`;
- `substore/frontend/` with the original Vite output;
- `runtime/node.exe`, downloaded from the official Node `v24.15.0` Windows x64 distribution and verified against the direct executable SHA-256 `3331e1ffe19874215472217c5e94f5a0c8f42e8c4ac7111d3937aa0ad5e9b4a5`.

Build-time `pnpm` and Node are required only on the developer/CI machine. The installed user needs neither Node nor pnpm.

### Backend lifecycle

Electron creates `%APPDATA%/VelvLens/sub-store-data` through `app.getPath('userData')`, starts the bundled `node.exe` with:

```text
SUB_STORE_BACKEND_API_HOST=127.0.0.1
SUB_STORE_BACKEND_API_PORT=0
SUB_STORE_BACKEND_MERGE=true
SUB_STORE_FRONTEND_BACKEND_PATH=/velvlens-api
SUB_STORE_FRONTEND_PATH=<resources>/substore/frontend
SUB_STORE_DATA_BASE_PATH=<userData>/sub-store-data
SUB_STORE_BACKEND_CUSTOM_NAME=VelvLens
```

Port `0` delegates allocation to Windows. Electron reads only the sanitized readiness line `[BACKEND] listening on 127.0.0.1:<port>` from the child output. It then loads `http://127.0.0.1:<port>/?magicpath=velvlens-api`; the original frontend resolves that magic path to the same-origin backend and uses its existing `/api/*`, `/download/*`, `/share/*`, and `/api/preview/*` endpoints.

The backend process is stopped on application quit and on retry before a new process is started. No backend listener binds `0.0.0.0`, and no public backend is used.

### Error recovery

The main process classifies startup failures without displaying raw backend output:

- missing bundled file;
- child process spawn failure;
- port/listen failure;
- child exit code before readiness;
- startup timeout.

The BrowserWindow shows `Не удалось запустить локальный resolver`, the category and safe detail, plus `Повторить запуск`. The preload bridge exposes only an `invokeRetry` action to the error page; it does not expose Node, filesystem, shell, or arbitrary IPC access to the upstream frontend.

## Electron security boundary

The BrowserWindow uses:

- `contextIsolation: true`;
- `nodeIntegration: false`;
- `sandbox: true` where supported;
- no unrestricted preload API;
- `setWindowOpenHandler` that permits only `http`/`https` external links through `shell.openExternal` and denies embedded external windows;
- `will-navigate` protection that keeps the main window on its dynamic localhost origin and sends external HTTP(S) navigation to the system browser;
- title override `VelvLens`;
- a conservative CSP for local scripts/styles, data/blob images and workers, same-origin API traffic, and HTTPS subscription/icon traffic, while allowing the upstream frontend's required `unsafe-eval` compatibility behavior.

The wrapper never logs subscription URLs or backend response bodies. It records only startup category, PID, selected localhost port, and readiness timing. Sub-Store's own runtime data stays under the per-user data directory.

## Build and packaging

Root `package.json` uses pinned `electron` and `electron-builder` dev dependencies. The build graph is:

```text
pnpm install --frozen-lockfile
pnpm build:substore
pnpm build:runtime
pnpm build:electron
```

`electron-builder` creates:

- a Windows NSIS installer `VelvLens-Setup.exe`;
- a Windows portable `VelvLens-portable.exe`.

The final package excludes git metadata, submodule working trees, package-manager caches, Rust targets, tests, and source-only artifacts. It includes the generated upstream runtime, Node license notice, Sub-Store backend AGPL license, Sub-Store frontend GPL license, exact commits, and source retrieval/build instructions sufficient for license compliance.

## Test strategy

The focused gates are separate from packaging and visual approval:

1. Node unit tests cover readiness-line parsing, safe error classification, external-navigation policy, and child-process cleanup.
2. `scripts/smoke-substore.js` launches the generated backend with a dynamic port and proves merged frontend `200`, magic-path `/api/utils/env` `200`, and clean shutdown.
3. The same smoke test creates a local fixture subscription through `/api/subs`, checks the server appears in `/api/subs`, and downloads the generated V2Ray output through `/download/<name>/V2Ray`; no private URL or token is stored.
4. Electron smoke launches the desktop main process, observes the BrowserWindow URL and title, confirms the backend child exists, and confirms it is gone after app close.
5. A clean-install script installs the built artifact into a fresh temporary directory, starts only the EXE, and repeats the backend/UI/fixture checks without a developer Node or pnpm command.
6. `electron-builder --win nsis portable` output is measured and recorded; artifact SHA-256 and Git HEAD are recorded in the final report.

The following remain explicit acceptance boundaries: upstream parser coverage is owned by Sub-Store; package build success is not live browser acceptance; fixture success is not evidence for arbitrary private subscription formats; and the final visual review is reported separately from automated smoke gates.

## Migration and deletion boundary

After the Electron smoke and clean-install gates pass, remove the Rust application and all now-unreachable Rust dependency/test surfaces:

- `src/resolver/`;
- `src/protocols/`;
- `src/share_uri/`;
- `src/export/`;
- `src/model/proxy_config.rs` and related resolver model code;
- `src/ui/`, egui/eframe code, QR and old clipboard/export flow;
- resolver-only Rust binaries, integration tests, fixtures, and Cargo dependencies;
- Rust-specific CI/build setup and old SubLens documentation.

The deletion is performed by reverse reachability from the new Electron entrypoints and then verified with `rg` for old module names and `cargo` metadata. No compatibility resolver is retained.

## Definition of done

The migration is complete only when a clean Windows EXE starts the pinned Sub-Store backend automatically on `127.0.0.1` with a dynamic port, opens the original frontend inside Electron, adds and displays a fixture subscription, exports a V2Ray result, shuts down the child backend with the app, packages Node without a separate user installation, contains the required license/source notices, and no Rust resolver/UI application remains in the repository.
