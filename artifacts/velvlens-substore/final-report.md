# VelvLens Sub-Store migration report

Generated: 2026-09-19T19:56:00Z
Git HEAD at evidence generation: `007181cfe7b5db77f0f29a80bccbe6fcab6494f0`

## Pinned embedding

- Backend: Sub-Store `8c2a695663b29a339f27651c6c683d54b04c7952`, version `2.39.9`, AGPL-3.0.
- Frontend: Sub-Store-Front-End `628403546cca2c00056cad35d5e285f9c9874b6d`, version `2.32.2`, GPL-3.0.
- Runtime: Node.js `24.15.0`, direct executable hash is recorded in `LICENSES/NODE-24.15.0.txt`.
- Staging manifest: `.build/substore/manifest.json`.

## Runtime flow

- Electron starts `resources/runtime/node.exe` with the upstream backend bundle at `resources/substore/backend/sub-store.bundle.js`.
- The backend is forced to `127.0.0.1` and port `0`; readiness yields a dynamic loopback port.
- The BrowserWindow loads `http://127.0.0.1:<dynamic-port>/?magicpath=velvlens-api`.
- Persistent data is under Electron `userData/sub-store-data`; no external Node.js or pnpm installation is required.

## Package artifacts

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| VelvLens-portable.exe | 137145387 | `c1a5df4eb838a7de7d4657ac6df42dc9be38ab2e32cb6564cdb7f766b49e19c6` |

The final `release` directory intentionally retains only the portable EXE;
installer and electron-builder staging outputs were removed from the
distribution directory after verification.

Package verification: PASS — the staged backend/frontend, Node runtime hash,
license notices, exact manifest pins, no `node_modules`, and no legacy Rust
executable/source entries were checked with `pnpm verify:package` after the
Windows build.

## Functional evidence

- Focused Electron tests: PASS (15/15) with `pnpm test:electron`.
- Sub-Store fixture smoke: PASS with VLESS, Base64, Hysteria2, JSON, and mixed
  fixtures; list retrieval and V2Ray export were verified with
  `pnpm smoke:substore`. Fixture export is decoded only in memory.
- Electron lifecycle smoke: PASS with `pnpm smoke:electron`; it observes
  BrowserWindow readiness and backend shutdown.
- Clean-install portable EXE: PASS (reported by the clean-install gate).
- Final visual/accessibility approval: separate human review gate, not inferred from smoke tests.

## Localhost and IPC feasibility audit

- The original frontend uses one Axios instance with a configurable `baseURL`
  and also contains direct Axios calls for backend checks, blob exports, and
  external URLs. Its API surface includes `/api/*`, `/download/*`, and
  `/share/*` behavior; this is not a single health endpoint that can be
  replaced transparently.
- The original Node backend registers the complete REST surface and calls
  `app.listen(host, port)`. The generated CommonJS bundle is a side-effecting
  application entrypoint and does not expose a request dispatcher or route
  registry for an Electron IPC adapter.
- Throwaway in-process probe: PASS for running the unchanged backend bundle in
  the Electron main process and receiving `200` from
  `/velvlens-api/api/utils/env`; the backend still opened a dynamic
  `127.0.0.1` listener. This proves only that a child `node.exe` can be removed
  while retaining loopback HTTP, not that the full UI contract is safe in
  process.
- Throwaway direct-resource probe: the original frontend loaded only its
  `file://` HTML shell; absolute `/index.js` and `/registerSW.js` resolved to
  the drive root and failed with `ERR_FILE_NOT_FOUND`, leaving `#app` empty.
- Decision: keep the current loopback transport and packaged Node runtime.
  A no-HTTP IPC variant would require changing the upstream backend entry and
  response lifecycle plus the frontend Axios adapter, direct Axios calls,
  blob/download links, share paths, and navigation/PWA assumptions. That is a
  large custom transport rewrite, so it is rejected in favor of the verified
  original Sub-Store behavior.

## Source and licensing

Composite notices and source retrieval/build instructions are in
`THIRD_PARTY_NOTICES.md`; shipped license texts are under `LICENSES/`.
The pinned upstream repositories are retained as git submodules for source
availability and reproducibility.
