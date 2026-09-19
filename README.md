# VelvLens

VelvLens is a standalone Windows Electron desktop application that embeds the
pinned Sub-Store frontend and backend. It opens the original Sub-Store UI in a
local BrowserWindow and starts the resolver locally; no external browser,
Node.js installation, pnpm installation, cloud backend, or second resolver is
required at runtime.

## Runtime flow

On launch Electron starts the packaged Node.js 24.15.0 executable with the
pinned Sub-Store backend bundle. The backend binds only to `127.0.0.1` and asks
the OS for a dynamic port. After the readiness line is received, the
BrowserWindow loads:

```text
http://127.0.0.1:<dynamic-port>/?magicpath=velvlens-api
```

Subscription data is kept below Electron's `userData/sub-store-data` directory.
If the backend cannot start, VelvLens shows a bounded retry page and does not
display raw process output or subscription data. External HTTP(S) links are
opened through the system browser; other navigation and new windows are
blocked.

## Use

1. Start VelvLens.
2. Add a subscription or local source in the embedded Sub-Store interface.
3. Review the resolved list and choose the required export format, such as
   V2Ray.

The packaged frontend/backend are upstream Sub-Store artifacts. VelvLens adds
only the Electron lifecycle, loopback boundary, retry UI, packaging, product
icon, and verification gates.

## Build from source

The repository pins the upstream sources as git submodules. On a build machine
with Node.js 24.15.0 and pnpm 11:

```text
git submodule update --init --recursive
pnpm install --frozen-lockfile
pnpm test:electron
pnpm build:substore
pnpm build:runtime
pnpm smoke:substore
pnpm smoke:electron
pnpm dist
pnpm verify:package
pnpm report
```

`pnpm dist` creates both `release/VelvLens Setup 0.2.0.exe` and
`release/VelvLens-portable.exe`. The portable executable contains Electron,
the verified Node runtime, the pinned Sub-Store artifacts, and license notices;
end users do not need Node or pnpm.

## Fixtures and verification

`tests/fixtures/substore-vless.txt` is synthetic and safe for local smoke
tests. `pnpm smoke:substore` exercises the embedded frontend, health endpoint,
subscription insertion, list retrieval, and V2Ray export without logging the
fixture body or writing decoded subscription data to disk.

The evidence report is generated at
`artifacts/velvlens-substore/final-report.md`. It records the Git HEAD, exact
upstream commits, runtime flow, installer/portable sizes and hashes, package
verification, and the boundaries between automated evidence, clean-install
testing, and final human visual review.

## Licensing and source availability

VelvLens's Electron shell is MIT-licensed. The embedded Sub-Store backend is
AGPL-3.0 and its frontend is GPL-3.0. Node.js licensing information is also
shipped. See `THIRD_PARTY_NOTICES.md` and `LICENSES/`; the exact upstream
repositories and commits remain available in `third_party/` for source
retrieval and reproducible builds.
