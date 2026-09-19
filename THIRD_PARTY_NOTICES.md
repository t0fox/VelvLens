# VelvLens third-party notices

VelvLens is a Windows Electron shell around the pinned Sub-Store frontend and
backend. The application does not modify the upstream frontend or backend
source. The repository keeps exact source gitlinks for reproducible builds and
ships the applicable license texts under `LICENSES/`.

## Sub-Store backend

- Repository: https://github.com/sub-store-org/Sub-Store
- Pinned commit: `8c2a695663b29a339f27651c6c683d54b04c7952`
- Release package version at that commit: `2.39.9`
- License: GNU Affero General Public License v3.0 (`AGPL-3.0`)
- License text: `LICENSES/SUB-STORE-AGPL-3.0.txt`
- Upstream license: https://github.com/sub-store-org/Sub-Store/blob/8c2a695663b29a339f27651c6c683d54b04c7952/LICENSE

The distributed backend is the upstream esbuild bundle generated with:

    pnpm install --frozen-lockfile
    pnpm bundle:esbuild

## Sub-Store frontend

- Repository: https://github.com/sub-store-org/Sub-Store-Front-End
- Pinned commit: `628403546cca2c00056cad35d5e285f9c9874b6d`
- Release package version at that commit: `2.31.2`
- License: GNU General Public License v3.0 (`GPL-3.0`)
- License text: `LICENSES/SUB-STORE-FRONTEND-GPL-3.0.txt`
- Upstream license: https://github.com/sub-store-org/Sub-Store-Front-End/blob/628403546cca2c00056cad35d5e285f9c9874b6d/LICENSE

The distributed frontend is the upstream Vite build generated with:

    pnpm install --frozen-lockfile
    pnpm build

## Node.js runtime

- Runtime: official Node.js `v24.15.0` Windows x64 `node.exe`
- Direct executable SHA-256: `3331e1ffe19874215472217c5e94f5a0c8f42e8c4ac7111d3937aa0ad5e9b4a5`
- License and third-party notices: `LICENSES/NODE-24.15.0.txt`
- Source: https://github.com/nodejs/node/tree/v24.15.0
- Official distribution: https://nodejs.org/dist/v24.15.0/

## Source retrieval

Initialize the exact upstream source gitlinks with:

    git submodule update --init --recursive

The package build consumes only generated files from the exact source commits;
it excludes the submodule working trees, package-manager caches, tests, and
other source-only material from the Windows package. Corresponding source and
build instructions remain available from this repository and the upstream
repositories above, as required by the applicable copyleft licenses.
