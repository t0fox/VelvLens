# VelvLens Sub-Store Electron Implementation Plan

> For agentic workers: REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

Goal: Replace the Rust/egui resolver application with a Windows Electron shell that packages pinned Sub-Store frontend/backend and a verified Node runtime.

Architecture: Two exact upstream git submodules are built at package time. The original Sub-Store backend bundle serves the original frontend in merge mode on 127.0.0.1 with port 0; Electron starts/stops that backend, loads ?magicpath=velvlens-api, enforces the browser boundary, and renders a safe retry page when startup fails. The legacy Rust resolver/UI is removed only after the new runtime and clean-install gates pass.

Tech Stack: Node.js 24.15.0 runtime, Electron 44.4.3, electron-builder 26.15.3, pnpm 11, upstream Sub-Store esbuild/Vite builds, Node node:test, Windows NSIS and portable targets.

Spec: docs/superpowers/specs/2026-09-19-velvlens-substore-electron-design.md

## Global Constraints

- Pin backend submodule to 8c2a695663b29a339f27651c6c683d54b04c7952 and frontend submodule to 628403546cca2c00056cad35d5e285f9c9874b6d.
- Do not add Rust, a second resolver, a second backend, a CGI service, or a custom protocol parser.
- Backend must bind only 127.0.0.1, use port 0, and persist only under Electron userData.
- BrowserWindow must use contextIsolation true, nodeIntegration false, sandbox where supported, and a minimal retry-only preload bridge.
- The package must include Node 24.15.0; the user must not install Node or pnpm.
- Do not log complete subscription URLs, tokens, backend bodies, or raw backend startup output.
- Preserve and ship Sub-Store AGPL-3.0, Sub-Store frontend GPL-3.0, Node licensing notices, exact commits, and source/build instructions.
- Keep package-build, backend smoke, Electron smoke, clean-install, and final visual review as separate evidence gates.
- Do not delete the old Rust resolver/UI until the new EXE has passed runtime and clean-install checks.

---

### Task 1: Add pinned upstream sources and composite license notices

Files:
- Create .gitmodules.
- Create third_party/sub-store as the backend source submodule.
- Create third_party/sub-store-frontend as the frontend source submodule.
- Create LICENSES/SUB-STORE-AGPL-3.0.txt.
- Create LICENSES/SUB-STORE-FRONTEND-GPL-3.0.txt.
- Create LICENSES/NODE-24.15.0.txt.
- Create THIRD_PARTY_NOTICES.md.
- Modify .gitignore.

Interfaces:
- The two submodule paths are initialized and pinned to exact commits.
- THIRD_PARTY_NOTICES.md names repositories, commits, licenses, runtime version, source retrieval, and build commands.

- [ ] Step 1: Verify branch and destinations.

    Run:
    
        git status --short --branch
        git ls-tree HEAD third_party/sub-store third_party/sub-store-frontend

    Expected: migration branch is clean and neither destination is already tracked.

- [ ] Step 2: Add exact submodules.

    Run:
    
        git submodule add https://github.com/sub-store-org/Sub-Store.git third_party/sub-store
        git submodule add https://github.com/sub-store-org/Sub-Store-Front-End.git third_party/sub-store-frontend
        git -C third_party/sub-store checkout 8c2a695663b29a339f27651c6c683d54b04c7952
        git -C third_party/sub-store-frontend checkout 628403546cca2c00056cad35d5e285f9c9874b6d

    Expected: .gitmodules and gitlink entries point to the exact commits; upstream files are unchanged.

- [ ] Step 3: Add notices and ignore generated output.

    THIRD_PARTY_NOTICES.md must include the exact backend/frontend URLs and SHAs, AGPL-3.0/GPL-3.0, Node 24.15.0, the Node URL, the source retrieval command git submodule update --init --recursive, and the two upstream build commands. Copy each upstream LICENSE into the corresponding LICENSES file and add Node's license/third-party URL. Add node_modules/, .build/, release/, artifacts/velvlens-substore/, and third_party/**/dist/ to .gitignore.

- [ ] Step 4: Verify pins and notices.

    Run:
    
        git -C third_party/sub-store rev-parse HEAD
        git -C third_party/sub-store-frontend rev-parse HEAD
        rg -n "8c2a695|628403546|AGPL-3.0|GPL-3.0|24.15.0" THIRD_PARTY_NOTICES.md LICENSES

    Expected: both exact SHAs and all license/runtime notices are present.

- [ ] Step 5: Commit.

        git add .gitmodules .gitignore third_party LICENSES THIRD_PARTY_NOTICES.md
        git commit -m "build: pin Sub-Store sources and runtime notices"

### Task 2: Create the Node/Electron project shell and verified runtime downloader

Files:
- Create package.json.
- Create electron-builder.yml.
- Create scripts/fetch-node-runtime.mjs.
- Create assets/icon.svg.
- Create tests/electron/runtime-assets.test.mjs.
- Modify .gitignore.

Interfaces:
- downloadVerified({ url, destination, sha256 }) streams a verified file through a temporary .part path.
- runtimePath({ packaged, resourcesPath, projectRoot }) returns the packaged or development node.exe path.
- package scripts provide start, build:substore, build:runtime, test:electron, smoke:substore, and dist.

- [ ] Step 1: Write failing runtime tests.

    Test matching and mismatching SHA-256 values using a temporary local HTTP server. Assert a mismatch throws and leaves no final file. Test runtimePath for packaged resourcesPath/runtime/node.exe and development .build/runtime/node.exe.

        node --test tests/electron/runtime-assets.test.mjs

    Expected: FAIL because the module does not exist.

- [ ] Step 2: Implement scripts/fetch-node-runtime.mjs.

    Download https://nodejs.org/dist/v24.15.0/win-x64/node.exe, verify SHA-256 3331e1ffe19874215472217c5e94f5a0c6d8e18c4ac7111d3937aa0ad5e9b4a5, then rename the verified temporary file to .build/runtime/node.exe. Reject non-2xx responses and never print response bodies.

- [ ] Step 3: Add package metadata and builder constraints.

    package.json pins Electron 44.4.3, electron-builder 26.15.3, packageManager pnpm@11.0.9, main electron/main.mjs, and uses:
    
        "start": "electron ."
        "build:substore": "node scripts/build-substore.mjs"
        "build:runtime": "node scripts/fetch-node-runtime.mjs"
        "test:electron": "node --test tests/electron/*.test.mjs"
        "smoke:substore": "node scripts/smoke-substore.mjs"
        "dist": "pnpm build:substore && pnpm build:runtime && electron-builder --win nsis portable"

    electron-builder.yml sets appId org.velvlens.desktop, productName VelvLens, asar true, NSIS and portable targets, extraResources for .build/substore and .build/runtime, and excludes third_party, target, tests, node_modules, and package-manager caches.

- [ ] Step 4: Add a 256px non-upstream product icon.

    Create a simple violet/white VelvLens SVG mark and configure builder to use it. Do not copy an upstream logo or redesign the Sub-Store interface.

- [ ] Step 5: Verify.

        pnpm install --frozen-lockfile
        pnpm test:electron
        pnpm build:runtime
        Get-FileHash .build/runtime/node.exe -Algorithm SHA256

    Expected: tests pass and the printed hash equals 3331e1ffe19874215472217c5e94f5a0c6d8e18c4ac7111d3937aa0ad5e9b4a5.

- [ ] Step 6: Commit.

        git add package.json pnpm-lock.yaml electron-builder.yml scripts/fetch-node-runtime.mjs assets tests/electron .gitignore
        git commit -m "build: add Electron packaging shell and Node runtime"

### Task 3: Build pinned Sub-Store artifacts reproducibly

Files:
- Create scripts/build-substore.mjs.
- Create tests/electron/substore-build.test.mjs.
- Modify package.json.
- Modify electron-builder.yml.

Interfaces:
- assertPinnedRepo(repoPath, expectedSha) rejects a missing or wrong submodule.
- runPnpm(cwd, args) propagates a nonzero process exit.
- buildSubStore({ root, backendSha, frontendSha }) writes .build/substore/backend/sub-store.bundle.js, backend metadata, and frontend dist.

- [ ] Step 1: Write failing pin/staging tests.

    Test both exact SHAs and require the staging contract: backend bundle, frontend index.html, at least one frontend chunk, and no node_modules below .build/substore.

        node --test tests/electron/substore-build.test.mjs

    Expected: FAIL because the build script and staging directory do not exist.

- [ ] Step 2: Implement the upstream build.

    Run from each submodule, without editing upstream source:
    
        pnpm install --frozen-lockfile
        pnpm bundle:esbuild
        pnpm install --frozen-lockfile
        pnpm build

    Copy only backend dist/sub-store.bundle.js, backend runtime-manifest.json, and all frontend dist files into .build/substore. Refuse to build before pin checks pass.

- [ ] Step 3: Write .build/substore/manifest.json.

    Include backend/frontend SHAs, package versions, Node tested version 24.15.0, UTC build time, and generated paths. This manifest is packaged and included in the final report.

- [ ] Step 4: Verify.

        pnpm build:substore
        node --test tests/electron/substore-build.test.mjs
        Get-ChildItem .build/substore -Recurse -File | Measure-Object Length -Sum

    Expected: upstream builds and staging tests pass; no node_modules appears in the staging tree.

- [ ] Step 5: Commit.

        git add scripts/build-substore.mjs tests/electron/substore-build.test.mjs package.json electron-builder.yml
        git commit -m "build: compile pinned Sub-Store artifacts"

### Task 4: Implement backend lifecycle and safe startup failures

Files:
- Create electron/backend.mjs.
- Create tests/electron/backend.test.mjs.

Interfaces:
- parseReadyPort(line) returns a number only for [BACKEND] listening on 127.0.0.1:<port>.
- classifyStartupFailure({ reason, code, signal }) returns safe code/message pairs.
- startBackend({ nodeBinary, backendEntry, frontendRoot, dataRoot, timeoutMs, spawnProcess }) resolves child, port, origin, and backendPath.
- stopBackend(child) resolves after bounded child shutdown.

- [ ] Step 1: Write failing lifecycle tests.

    Cover a valid loopback readiness line, reject 0.0.0.0, classify timeout as STARTUP_TIMEOUT, classify pre-ready exit as BACKEND_EXITED, and assert child environment includes:
    
        SUB_STORE_BACKEND_API_HOST=127.0.0.1
        SUB_STORE_BACKEND_API_PORT=0
        SUB_STORE_BACKEND_MERGE=true
        SUB_STORE_FRONTEND_BACKEND_PATH=/velvlens-api

    Use a fake EventEmitter child and ensure stdout/stderr contents never appear in rejection messages.

        node --test tests/electron/backend.test.mjs

    Expected: FAIL because lifecycle functions do not exist.

- [ ] Step 2: Implement bounded readiness.

    Spawn bundled node.exe with windowsHide true, keep output in memory only, parse stdout/stderr for the readiness line, apply a 30-second timeout, and reject on spawn error, pre-ready exit, invalid port, or timeout.

- [ ] Step 3: Implement clean shutdown.

    Call child.kill(), wait at most five seconds for close, and resolve when already dead. Do not use broad process-name termination. Keep one child reference in the main process.

- [ ] Step 4: Verify and commit.

        node --test tests/electron/backend.test.mjs
        git add electron/backend.mjs tests/electron/backend.test.mjs
        git commit -m "feat: manage local Sub-Store backend lifecycle"

### Task 5: Implement secure BrowserWindow and retry page

Files:
- Create electron/main.mjs.
- Create electron/preload.mjs.
- Create electron/backend-error.html.
- Create tests/electron/security.test.mjs.

Interfaces:
- createMainWindow() returns the single BrowserWindow.
- loadSubStore(win, runtime) starts the backend and loads the dynamic same-origin URL.
- showBackendError(win, failure) renders the safe Russian failure page.
- preload exposes only window.velvLens.retryBackend().

- [ ] Step 1: Write failing security tests.

    Static assertions require contextIsolation true, nodeIntegration false, sandbox true, setWindowOpenHandler, will-navigate, and shell.openExternal. Preload must expose only retryBackend. URL policy must accept the dynamic 127.0.0.1 origin and reject another origin.

        node --test tests/electron/security.test.mjs

    Expected: FAIL because Electron files do not exist.

- [ ] Step 2: Implement main process.

    Create one BrowserWindow with title VelvLens, minimum size 900x650, secure webPreferences, and preload. Start backend before loading:
    
        http://127.0.0.1:<port>/?magicpath=velvlens-api

    Prevent page title changes. Route only external http/https links to shell.openExternal and deny all other protocols/windows. Stop the backend on before-quit and after window close.

- [ ] Step 3: Implement preload and error page.

    Allowlist one ipcMain handler backend:retry. The error page shows exactly Не удалось запустить локальный resolver, a safe category/detail, and Повторить запуск. Never interpolate backend output or subscription URLs into HTML.

- [ ] Step 4: Add CSP.

    For the local app session, set:
    
        default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self' https:; worker-src 'self' blob:; frame-src 'none'; object-src 'none'; base-uri 'self'; form-action 'self'

    Confirm lazy chunks and remote subscription/icon requests remain functional.

- [ ] Step 5: Verify and commit.

        node --test tests/electron/security.test.mjs tests/electron/backend.test.mjs
        git add electron tests/electron/security.test.mjs
        git commit -m "feat: open Sub-Store inside secure Electron window"

### Task 6: Add backend fixture and Electron lifecycle smoke tests

Files:
- Create scripts/smoke-substore.mjs.
- Create scripts/smoke-electron.mjs.
- Create tests/fixtures/substore-vless.txt.
- Create artifacts/velvlens-substore/README.md.
- Modify package.json.

Interfaces:
- smoke-substore.mjs exits nonzero unless merged frontend, magic-path health, fixture list, V2Ray download, and child shutdown pass.
- smoke-electron.mjs exits nonzero unless BrowserWindow and backend lifecycle are observed.

- [ ] Step 1: Add deterministic fixture.

    Use UUID 00000000-0000-0000-0000-000000000001, example.com:443, websocket /ws, display name Fixture. Do not use private URLs, tokens, or credentials.

- [ ] Step 2: Implement Sub-Store smoke.

    Assert:
    
        GET / -> 200 text/html
        GET /velvlens-api/api/utils/env -> 200 status=success
        GET /?magicpath=velvlens-api -> 200
        POST /velvlens-api/api/subs -> 201 status=success
        GET /velvlens-api/api/subs -> fixture present
        GET /velvlens-api/download/fixture/V2Ray -> 200 non-empty text/plain

    Decode output only in memory and assert vless:// and example.com; never write it to a report.

- [ ] Step 3: Implement Electron smoke.

    Launch Electron with a smoke environment flag that emits a bounded readiness marker to the parent. Observe the BrowserWindow URL/title, then close the app and assert the backend PID is gone. Do not open an external browser.

- [ ] Step 4: Run and commit.

        pnpm build:substore
        pnpm smoke:substore
        node scripts/smoke-electron.mjs
        git add scripts tests/fixtures artifacts/velvlens-substore package.json
        git commit -m "test: verify Sub-Store conversion and Electron lifecycle"

### Task 7: Add CI, package verification, and final evidence generation

Files:
- Modify .github/workflows/ci.yml.
- Create scripts/verify-package.mjs.
- Create scripts/write-final-report.mjs.
- Modify package.json.
- Modify electron-builder.yml.

Interfaces:
- verify-package.mjs checks packaged resources, exact manifest, Node hash, license files, no node_modules, and no Rust executable.
- write-final-report.mjs writes artifacts/velvlens-substore/final-report.md with commits, embedding layout, endpoint evidence, package sizes, hashes, and Git HEAD.

- [ ] Step 1: Configure Windows CI.

    Use checkout with submodules recursive, Node 24.15.0, Corepack pnpm 11.0.9, frozen install, Electron tests, Sub-Store build, smoke test, and pnpm dist.

- [ ] Step 2: Verify package contents.

    Require resources/substore/backend/sub-store.bundle.js, resources/substore/frontend/index.html, resources/runtime/node.exe, both license files, and THIRD_PARTY_NOTICES.md. Reject node_modules, src/resolver, sublens.exe, 0.0.0.0 settings, or missing pin metadata.

- [ ] Step 3: Build both Windows targets.

        pnpm dist

    Expected: release/VelvLens-Setup.exe and release/VelvLens-portable.exe exist.

- [ ] Step 4: Record evidence.

    Run Get-FileHash release/*.exe -Algorithm SHA256, measure sizes, record git rev-parse HEAD, and write the final report without user subscription data.

- [ ] Step 5: Commit.

        git add .github/workflows/ci.yml scripts/verify-package.mjs scripts/write-final-report.mjs package.json electron-builder.yml
        git commit -m "ci: build and verify standalone Windows artifacts"

### Task 8: Delete the superseded Rust application after green new-runtime gates

Files:
- Delete Rust resolver/UI/model/export/protocol source and resolver-only binaries.
- Delete resolver/share/export/UI Rust tests and fixtures.
- Delete Cargo.toml and Cargo.lock after replacement docs exist.
- Modify README.md and .github/workflows/ci.yml.
- Delete DESIGN.md and old SubLens docs when unreferenced.

- [ ] Step 1: Require green new gates.

    Do not delete Rust code until pnpm test:electron, pnpm smoke:substore, Electron smoke, pnpm dist, verify-package, and clean-install portable EXE checks pass.

- [ ] Step 2: Derive the deletion set.

        rg -n "sublens|resolver|protocols|share_uri|ProxyConfig|eframe|egui|Cargo" --glob '!docs/superpowers/specs/**' --glob '!docs/superpowers/plans/**'

    Delete only files whose callers are the superseded Rust entrypoints. Preserve new smoke fixtures, upstream sources, license notices, and Electron scripts.

- [ ] Step 3: Verify no parallel resolver.

        rg --files src tests Cargo.toml Cargo.lock 2>$null
        rg -n "src/resolver|src/protocols|src/share_uri|ProxyConfig|eframe|egui|sublens.exe|cargo test" README.md .github package.json electron scripts THIRD_PARTY_NOTICES.md

    Expected: no Rust product files remain and docs point to Electron/Sub-Store.

- [ ] Step 4: Run final gates and commit.

        pnpm install --frozen-lockfile
        pnpm test:electron
        pnpm build:substore
        pnpm smoke:substore
        pnpm dist
        node scripts/verify-package.mjs
        git diff --check
        git commit -m "feat: replace Rust resolver with Sub-Store desktop app"

### Task 9: Clean-install, final report, and delivery audit

Files:
- Modify artifacts/velvlens-substore/final-report.md.
- Modify README.md if final artifact names or measured sizes differ.

- [ ] Step 1: Run clean-install.

    From a fresh temporary directory, use only the portable EXE. Confirm no developer node.exe, pnpm, terminal, external browser, or manually configured backend is needed. Verify localhost backend, UI title/window, fixture add/list, V2Ray export, and child shutdown.

- [ ] Step 2: Record exact evidence.

    The report must contain backend/frontend commits and versions, embedding paths, host/port/magicpath behavior, removed Rust files/dependencies, portable and installer sizes/hashes, clean-install result, fixture list/export result, license/source changes, and Git HEAD.

- [ ] Step 3: Audit the Definition of Done.

    Compare current filesystem and report against every requirement in the design. Do not claim GREEN for unverified private formats, final visual approval, or external-browser absence without corresponding evidence.

- [ ] Step 4: Run finishing-a-development-branch.

    Inspect the final diff, keep separate test and visual-approval boundaries, and report exact branch/commit/artifacts plus any remaining human visual review gate.
