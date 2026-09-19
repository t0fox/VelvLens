import { execFile, spawn } from 'node:child_process';
import { cp, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const BACKEND_SHA = '8c2a695663b29a339f27651c6c683d54b04c7952';
const FRONTEND_SHA = '628403546cca2c00056cad35d5e285f9c9874b6d';
const NODE_VERSION = '24.15.0';

async function gitHead(repoPath) {
  const { stdout } = await execFileAsync('git', ['-C', repoPath, 'rev-parse', 'HEAD'], {
    cwd: projectRoot,
    windowsHide: true,
  });
  return stdout.trim();
}

export async function assertPinnedRepo(repoPath, expectedSha) {
  let actual;
  try {
    actual = await gitHead(resolve(projectRoot, repoPath));
  } catch (error) {
    throw new Error(`Unable to read upstream pin at ${repoPath}: ${error.message}`);
  }
  if (actual !== expectedSha) {
    throw new Error(`Upstream repo ${repoPath} has ${actual}, expected pin ${expectedSha}`);
  }
  return actual;
}

export function runPnpm(cwd, args) {
  const command = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, {
      cwd,
      stdio: 'inherit',
      windowsHide: true,
      shell: process.platform === 'win32',
    });
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`pnpm ${args.join(' ')} failed with ${signal ?? `exit ${code}`}`));
    });
  });
}

async function hasNodeModules(root) {
  const entries = await readdir(root, { recursive: true });
  return entries.some((entry) => entry.split(/[\\/]/u).includes('node_modules'));
}

export async function validateStaging(stageRoot) {
  const required = [
    join(stageRoot, 'backend', 'sub-store.bundle.js'),
    join(stageRoot, 'backend', 'runtime-manifest.json'),
    join(stageRoot, 'frontend', 'index.html'),
    join(stageRoot, 'manifest.json'),
  ];
  for (const filePath of required) {
    try {
      await readFile(filePath);
    } catch {
      throw new Error(`Sub-Store staging is missing ${relative(stageRoot, filePath)}`);
    }
  }

  const frontendEntries = await readdir(join(stageRoot, 'frontend'), { recursive: true });
  if (!frontendEntries.some((entry) => entry !== 'index.html' && /\.(?:js|css)$/u.test(entry))) {
    throw new Error('Sub-Store staging is missing a frontend chunk');
  }
  if (await hasNodeModules(stageRoot)) throw new Error('Sub-Store staging must not contain node_modules');
  return true;
}

export async function buildSubStore({
  root = projectRoot,
  backendSha = BACKEND_SHA,
  frontendSha = FRONTEND_SHA,
  runPnpmImpl = runPnpm,
} = {}) {
  const backendRepo = resolve(root, 'third_party/sub-store');
  const backendRoot = join(backendRepo, 'backend');
  const frontendRepo = resolve(root, 'third_party/sub-store-frontend');
  const frontendRoot = frontendRepo;
  await assertPinnedRepo(backendRepo, backendSha);
  await assertPinnedRepo(frontendRepo, frontendSha);

  const stageRoot = resolve(root, '.build/substore');
  await rm(stageRoot, { recursive: true, force: true });
  await runPnpmImpl(backendRoot, ['install', '--frozen-lockfile']);
  await runPnpmImpl(backendRoot, ['bundle:esbuild']);
  await runPnpmImpl(frontendRoot, ['install', '--frozen-lockfile']);
  await runPnpmImpl(frontendRoot, ['build']);

  await mkdir(join(stageRoot, 'backend'), { recursive: true });
  await cp(join(backendRoot, 'dist', 'sub-store.bundle.js'), join(stageRoot, 'backend', 'sub-store.bundle.js'));
  await cp(join(backendRoot, 'dist', 'runtime-manifest.json'), join(stageRoot, 'backend', 'runtime-manifest.json'));
  await cp(join(frontendRoot, 'dist'), join(stageRoot, 'frontend'), { recursive: true });

  const backendPackage = JSON.parse(await readFile(join(backendRoot, 'package.json'), 'utf8'));
  const frontendPackage = JSON.parse(await readFile(join(frontendRoot, 'package.json'), 'utf8'));
  const manifest = {
    backend: { repository: 'sub-store-org/Sub-Store', sha: backendSha, version: backendPackage.version },
    frontend: {
      repository: 'sub-store-org/Sub-Store-Front-End',
      sha: frontendSha,
      version: frontendPackage.version,
    },
    node: { version: NODE_VERSION },
    builtAt: new Date().toISOString(),
    generated: {
      backendBundle: 'backend/sub-store.bundle.js',
      backendRuntimeManifest: 'backend/runtime-manifest.json',
      frontend: 'frontend',
    },
  };
  await writeFile(join(stageRoot, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  await validateStaging(stageRoot);
  return { stageRoot, manifest };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = await buildSubStore();
  console.log(`Staged Sub-Store ${result.manifest.backend.sha} / ${result.manifest.frontend.sha}`);
}

export { BACKEND_SHA, FRONTEND_SHA, NODE_VERSION };
