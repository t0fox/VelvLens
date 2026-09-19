# VelvLens Sub-Store migration report

Generated: 2026-09-19T16:10:39.892Z
Git HEAD at evidence generation: `55e98a5a7bb980f919e7cc1a2877bb6775a82ee7`

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
| VelvLens Setup 0.2.0.exe | 137371902 | `557f0bf3f68fb36666efaad547975474861f31766f8dd21ccd5bfbc89306d112` |
| VelvLens-portable.exe | 137145305 | `995635fd2d53f9a68e958f13ee663526490a4d152798ee1aa28dc6770b769873` |

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

## Source and licensing

Composite notices and source retrieval/build instructions are in
`THIRD_PARTY_NOTICES.md`; shipped license texts are under `LICENSES/`.
The pinned upstream repositories are retained as git submodules for source
availability and reproducibility.
