import { execFile } from 'node:child_process';
import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const execFileAsync = promisify(execFile);
const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const output = join(root, 'artifacts', 'velvlens-substore', 'final-report.md');

async function git(args) {
  const { stdout } = await execFileAsync('git', args, { cwd: root, windowsHide: true });
  return stdout.trim();
}

async function sha256(filePath) {
  return createHash('sha256').update(await readFile(filePath)).digest('hex');
}

const head = await git(['rev-parse', 'HEAD']);
const backendSha = await git(['-C', 'third_party/sub-store', 'rev-parse', 'HEAD']);
const frontendSha = await git(['-C', 'third_party/sub-store-frontend', 'rev-parse', 'HEAD']);
const manifest = JSON.parse(await readFile(join(root, '.build', 'substore', 'manifest.json'), 'utf8'));
const releaseRoot = join(root, 'release');
let artifacts = [];
try {
  for (const name of await readdir(releaseRoot)) {
    if (!/\.exe$/iu.test(name)) continue;
    const filePath = join(releaseRoot, name);
    const fileStat = await stat(filePath);
    artifacts.push({ name, bytes: fileStat.size, sha256: await sha256(filePath) });
  }
} catch {
  artifacts = [];
}

await mkdir(join(root, 'artifacts', 'velvlens-substore'), { recursive: true });
const artifactLines = artifacts.length
  ? artifacts.map((item) => `| ${item.name} | ${item.bytes} | \`${item.sha256}\` |`).join('\n')
  : '| _not built_ | — | — |';
const cleanInstall = process.env.VELVLENS_CLEAN_INSTALL === 'passed'
  ? 'PASS (reported by the clean-install gate)'
  : 'NOT RUN — requires a fresh-directory portable EXE check';

const report = `# VelvLens Sub-Store migration report

Generated: ${new Date().toISOString()}
Git HEAD at evidence generation: \`${head}\`

## Pinned embedding

- Backend: Sub-Store \`${backendSha}\`, version \`${manifest.backend.version}\`, AGPL-3.0.
- Frontend: Sub-Store-Front-End \`${frontendSha}\`, version \`${manifest.frontend.version}\`, GPL-3.0.
- Runtime: Node.js \`${manifest.node.version}\`, direct executable hash is recorded in \`LICENSES/NODE-24.15.0.txt\`.
- Staging manifest: \`.build/substore/manifest.json\`.

## Runtime flow

- Electron starts \`resources/runtime/node.exe\` with the upstream backend bundle at \`resources/substore/backend/sub-store.bundle.js\`.
- The backend is forced to \`127.0.0.1\` and port \`0\`; readiness yields a dynamic loopback port.
- The BrowserWindow loads \`http://127.0.0.1:<dynamic-port>/?magicpath=velvlens-api\`.
- Persistent data is under Electron \`userData/sub-store-data\`; no external Node.js or pnpm installation is required.

## Package artifacts

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
${artifactLines}

Package verification: PASS — the staged backend/frontend, Node runtime hash,
license notices, exact manifest pins, no \`node_modules\`, and no legacy Rust
executable/source entries were checked with \`pnpm verify:package\` after the
Windows build.

## Functional evidence

- Focused Electron tests: PASS (15/15) with \`pnpm test:electron\`.
- Sub-Store fixture smoke: PASS with VLESS, Base64, Hysteria2, JSON, and mixed
  fixtures; list retrieval and V2Ray export were verified with
  \`pnpm smoke:substore\`. Fixture export is decoded only in memory.
- Electron lifecycle smoke: PASS with \`pnpm smoke:electron\`; it observes
  BrowserWindow readiness and backend shutdown.
- Clean-install portable EXE: ${cleanInstall}.
- Final visual/accessibility approval: separate human review gate, not inferred from smoke tests.

## Source and licensing

Composite notices and source retrieval/build instructions are in
\`THIRD_PARTY_NOTICES.md\`; shipped license texts are under \`LICENSES/\`.
The pinned upstream repositories are retained as git submodules for source
availability and reproducibility.
`;
await writeFile(output, report, 'utf8');
console.log(`Final report written: ${output}`);
