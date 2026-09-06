import { createHash } from 'node:crypto';
import { cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs';
import { basename, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { command } from './process-tools.mjs';

const excluded = new Set(['.git', 'target', 'bin', 'obj', '.build', '.run']);
const sha = value => createHash('sha256').update(value).digest('hex');
const slash = path => path.split(sep).join('/');

export function sourceFingerprint(roots) {
  const entries = [];
  const visit = (label, root, path) => {
    if (!existsSync(path) || excluded.has(basename(path))) return;
    const stat = lstatSync(path);
    if (stat.isSymbolicLink()) throw new Error(`Build input symlinks must be resolved explicitly: ${path}`);
    if (stat.isDirectory()) {
      for (const name of readdirSync(path).sort()) visit(label, root, join(path, name));
    } else if (stat.isFile()) entries.push(`${label}/${slash(relative(root, path))}\0${sha(readFileSync(path))}`);
  };
  for (const [label, root] of Object.entries(roots).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) {
    const paths = label === 'engine'
      ? ['engine/native', 'engine/abi', 'eng/dev-build.mjs', 'eng/process-tools.mjs']
      : ['Cargo.toml', 'Cargo.lock', 'crates'];
    for (const path of [...paths, '.cargo', 'rust-toolchain', 'rust-toolchain.toml']) visit(label, root, join(root, path));
  }
  return sha(entries.sort().join('\n'));
}

export function buildIdentity(sourceSha256, inputs) {
  // Absolute checkout/worktree paths are deliberately absent. Target and flags are NOT absent.
  return sha(JSON.stringify({ sourceSha256, ...inputs })).slice(0, 32);
}

export function rewriteDependencyRoots(manifest, roots) {
  // Preserve all original dependency properties; rewrite only the checkout root segment.
  return manifest.replace(/path\s*=\s*"([^"\n]*?)(LumioNativeCore|LumioVoxelEngine)\/([^"\n]+)"/g,
    (_match, _prefix, repo, suffix) => `path = ${JSON.stringify(`${slash(roots[repo])}/${suffix}`)}`);
}

export function buildNative({ root, nativeCoreRoot, voxelRoot, configuration = 'debug' }) {
  if (!['linux', 'win32'].includes(process.platform) || process.arch !== 'x64') {
    throw new Error('BLOCKED_ENV: the current native ABI verification profiles are Linux x64 and Windows x64.');
  }
  if (!['debug', 'release'].includes(configuration)) throw new Error('Configuration must be debug or release.');
  root = realpathSync(root);
  nativeCoreRoot = realpathSync(nativeCoreRoot);
  voxelRoot = realpathSync(voxelRoot);
  const rustc = command('rustc', ['--version', '--verbose'], { cwd: root }).trim();
  const cargo = command('cargo', ['--version'], { cwd: root }).trim();
  const host = /^host: (.+)$/m.exec(rustc)?.[1];
  if (!host) throw new Error('Cannot determine the Rust host target.');
  const target = process.env.CARGO_BUILD_TARGET || host;
  if (target !== host) throw new Error('BLOCKED_ENV: dev-run verifies native host builds, not cross-compiled artifacts.');
  const flags = Object.fromEntries(['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_PROFILE_RELEASE_LTO', 'CC', 'CFLAGS'].map(key => [key, process.env[key] ?? '']));
  const roots = { engine: root, nativeCore: nativeCoreRoot, voxel: voxelRoot };
  const sourceSha256 = sourceFingerprint(roots);
  const inputs = { configuration, target, rustc, cargo, flags };
  const buildId = buildIdentity(sourceSha256, inputs);
  const abiHash = sha(readFileSync(join(root, 'engine/abi/native-abi.json')));
  const buildRoot = join(root, '.build');
  mkdirSync(buildRoot, { recursive: true });
  // Mirror the engine/ layout: sdk-native reaches engine/abi via include_str!("../../../../abi/...").
  const workspace = mkdtempSync(join(buildRoot, 'native-workspace-'));
  const nativeWorkspace = join(workspace, 'native');
  cpSync(join(root, 'engine/native'), nativeWorkspace, {
    recursive: true, filter: path => !excluded.has(basename(path)),
  });
  cpSync(join(root, 'engine/abi'), join(workspace, 'abi'), { recursive: true });
  const sdkManifest = join(nativeWorkspace, 'modules/sdk-native/Cargo.toml');
  const manifest = readFileSync(sdkManifest, 'utf8');
  const rewritten = rewriteDependencyRoots(manifest, { LumioNativeCore: nativeCoreRoot, LumioVoxelEngine: voxelRoot });
  if (rewritten === manifest) throw new Error('SDK native manifest did not resolve any external dependency roots.');
  writeFileSync(sdkManifest, rewritten);
  const targetDir = join(buildRoot, 'native-target');
  const nativeManifest = join(nativeWorkspace, 'Cargo.toml');
  const env = { ...process.env, LUMIO_BUILD_ID: buildId, LUMIO_ABI_HASH: abiHash };
  command('cargo', ['build', '--locked', '--manifest-path', nativeManifest, '-p', 'lumio-engine-native', '--target', target,
    '--target-dir', targetDir, ...(configuration === 'release' ? ['--release'] : [])], { cwd: root, env, log: join(nativeWorkspace, 'build.log') });
  if (sourceFingerprint(roots) !== sourceSha256) throw new Error('Inputs changed during native build. Retry in an isolated worktree; no artifact was published.');
  const nativeName = process.platform === 'win32' ? 'lumio_engine_native.dll' : 'liblumio_engine_native.so';
  const builtPath = join(targetDir, target, configuration, nativeName);
  if (!existsSync(builtPath)) throw new Error(`Native output not found: ${builtPath}`);
  const platform = process.platform === 'win32' ? 'win-x64' : 'linux-x64';
  const stageParent = join(root, '.run', buildId, platform);
  mkdirSync(stageParent, { recursive: true });
  const stage = mkdtempSync(join(stageParent, 'run-'));
  const nativePath = join(stage, nativeName);
  cpSync(builtPath, nativePath);
  const binarySha256 = sha(readFileSync(nativePath));
  const info = { buildId, sourceSha256, abiHash, binarySha256, platform, nativePath, configuration,
    nativeManifest, targetDir, buildInputs: inputs };
  writeFileSync(join(stage, 'build-info.json'), JSON.stringify(info, null, 2) + '\n');
  console.log(`BUILD_ID=${buildId}\nABI_HASH=${abiHash}\nNATIVE_PATH=${nativePath}\nBINARY_SHA256=${binarySha256}\nNATIVE_MANIFEST=${nativeManifest}`);
  return info;
}

function main() {
  const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
  const values = {};
  const argv = process.argv.slice(2);
  for (let i = 0; i < argv.length; i += 2) {
    if (!['--native-core-root', '--voxel-root', '--configuration'].includes(argv[i]) || !argv[i + 1] || argv[i] in values) {
      throw new Error('usage: node eng/dev-build.mjs [--native-core-root path] [--voxel-root path] [--configuration debug|release]');
    }
    values[argv[i]] = argv[i + 1];
  }
  buildNative({ root, nativeCoreRoot: values['--native-core-root'] || process.env.NATIVE_CORE_ROOT || join(root, '../LumioNativeCore'),
    voxelRoot: values['--voxel-root'] || process.env.VOXEL_ROOT || join(root, '../LumioVoxelEngine'),
    configuration: values['--configuration'] || process.env.CONFIGURATION || 'debug' });
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); } catch (error) { console.error(`${error.code === 'ENOENT' ? 'BLOCKED_ENV' : 'FAIL'}: ${error.message}`); process.exitCode = error.code === 'ENOENT' ? 2 : 1; }
}
