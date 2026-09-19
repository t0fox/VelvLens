# VelvLens Sub-Store migration report

Generated: 2026-09-19T15:57:22.756Z
Git HEAD: `3f9da989e6344935aaf48a0bacde8ab8fd414208`

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
| VelvLens Setup 0.2.0.exe | 137371865 | `4c73c7df38c713f26654abae3ebae3af0c08a8e9e77a3fee52f9ea89d5c41c82` |
| VelvLens-portable.exe | 137145267 | `9b2ceca6ddee71f32651bd0ff1fea84384b2c28b889f18e2d963b6cba7ca1624` |

Package verification requires the staged backend/frontend, Node runtime hash,
license notices, exact manifest pins, no `node_modules`, and no legacy Rust
executable/source entries in the packaged app. It is run with
`pnpm verify:package` after the Windows build.

## Functional evidence

- Focused Electron tests: run with `pnpm test:electron`.
- Sub-Store fixture smoke: run with `pnpm smoke:substore`; fixture export is decoded only in memory.
- Electron lifecycle smoke: run with `pnpm smoke:electron`; it observes BrowserWindow readiness and backend shutdown.
- Clean-install portable EXE: NOT RUN — requires a fresh-directory portable EXE check.
- Final visual/accessibility approval: separate human review gate, not inferred from smoke tests.

## Source and licensing

Composite notices and source retrieval/build instructions are in
`THIRD_PARTY_NOTICES.md`; shipped license texts are under `LICENSES/`.
The pinned upstream repositories are retained as git submodules for source
availability and reproducibility.
