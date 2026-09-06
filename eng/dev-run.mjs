import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildNative } from './dev-build.mjs';
import { command, startLogged, assertAlive, waitExit, forceCleanup } from './process-tools.mjs';
import { readNdjson, until, openBrowserPeer, verifyHelloEvidence } from './hello-smoke.mjs';

const hashFile = path => createHash('sha256').update(readFileSync(path)).digest('hex');
function json(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function pendingJson(path) {
  try { return json(path); } catch (error) { if (error.code === 'ENOENT' || error instanceof SyntaxError) return null; throw error; }
}

export function managedBuildPlan(roots, evidence, verify) {
  const builds = [
    { project: join(roots.LumioGameRuntime, 'modules/hello/entry'), output: join(evidence, 'runtime') },
    { project: join(roots.LumioClient, 'modules/bot/host/Lumio.Client.Bot.Host.csproj'), output: join(evidence, 'foundation') },
  ];
  if (verify) builds.push({ project: join(roots.LumioClient, 'modules/hello/host'), output: join(evidence, 'hello-bot') });
  return builds.map(item => ({ ...item, args: ['build', item.project, '--configuration', 'Debug', '--output', item.output, '--nologo'] }));
}

export function parseNativeProof(line) {
  const match = /^ENGINE_NATIVE path=(.*?) buildId=([0-9a-f]+) abiHash=([0-9a-f]+) binarySha256=([0-9a-f]+)\s*$/i.exec(line);
  if (!match) throw new Error(`Invalid native proof: ${line}`);
  return { nativePath: match[1], buildId: match[2].toLowerCase(), abiHash: match[3].toLowerCase(), binarySha256: match[4].toLowerCase() };
}

function hostFxr(root) {
  if (process.env.LUMIO_HOSTFXR) return realpathSync(process.env.LUMIO_HOSTFXR);
  const roots = new Set([process.env.DOTNET_ROOT, process.env.DOTNET_ROOT_X64].filter(Boolean));
  const runtimes = command('dotnet', ['--list-runtimes'], { cwd: root });
  for (const line of runtimes.split('\n')) {
    const path = /\[([^\]]+)\]/.exec(line)?.[1];
    if (path) roots.add(resolve(path, '../..'));
  }
  const name = process.platform === 'win32' ? 'hostfxr.dll' : 'libhostfxr.so';
  for (const directory of roots) {
    const fxr = join(directory, 'host/fxr');
    if (!existsSync(fxr)) continue;
    for (const version of readdirSync(fxr).sort((a, b) => b.localeCompare(a, 'en', { numeric: true }))) {
      const path = join(fxr, version, name);
      if (existsSync(path)) return realpathSync(path);
    }
  }
  throw new Error('BLOCKED_ENV: hostfxr not found; set DOTNET_ROOT or LUMIO_HOSTFXR.');
}

function sources(roots, root) {
  return Object.fromEntries(Object.entries({ LumioGameEngine: root, ...roots }).map(([name, path]) => {
    const commit = command('git', ['rev-parse', 'HEAD'], { cwd: path }).trim();
    const dirty = command('git', ['status', '--porcelain'], { cwd: path }).trim() !== '';
    return [name, { commit, dirty, path }];
  }));
}

async function shutdown(server) {
  assertAlive(server);
  server.child.stdin.end('shutdown\n');
  await waitExit(server);
}

async function round(context, number, verify, keepRunning) {
  const { root, evidence, native, serverExe, hostfxr, contractPath, contract, runtimeDll, runtimeConfig, helloDll } = context;
  const dir = join(evidence, `round-${number}`);
  mkdirSync(dir);
  const auditPath = join(dir, 'audit.ndjson');
  const readyPath = join(dir, 'ready.json');
  const server = startLogged(serverExe, [
    '--engine-native', native.nativePath, '--hostfxr', hostfxr,
    '--runtime-config', runtimeConfig, '--assembly', runtimeDll,
    '--entry-type', 'Lumio.GameRuntime.HelloEntry.HelloEntry, Lumio.GameRuntime.HelloEntry',
    '--entry-method', 'LumioHelloEntry', '--wire-contract', contractPath,
    '--audit-file', auditPath, '--ready-file', readyPath,
  ], { cwd: root, log: join(dir, 'server.log') });
  let bot, peer;
  const interrupt = () => { if (!server.closed) server.child.stdin.end('shutdown\n'); };
  process.once('SIGINT', interrupt);
  process.once('SIGTERM', interrupt);
  try {
    const ready = await until(() => pendingJson(readyPath), 'Rust DS readiness', 30000, () => assertAlive(server));
    assert.equal(ready.contractId, contract.contractId);
    assert.equal(ready.pid, server.child.pid);
    assert.ok(Number.isInteger(ready.port) && ready.port > 0 && ready.port <= 65535);
    console.log(`SERVER SERVER_READY ${JSON.stringify(ready)}`);
    if (keepRunning) {
      console.log(`HOSTS_RUNNING serverPid=${ready.pid} evidence=${dir}`);
      await server.done;
      await waitExit(server);
      return null;
    }
    if (!verify) { await shutdown(server); return null; }
    const resultPath = join(dir, 'bot-result.json');
    const url = `ws://127.0.0.1:${ready.port}/`;
    bot = startLogged('dotnet', [helloDll, '--url', url, '--role', 'bot', '--contract', contractPath,
      '--trace', join(dir, 'bot.ndjson'), '--result', resultPath], { cwd: root, log: join(dir, 'bot.log') });
    const guard = () => { assertAlive(server); assertAlive(bot); };
    await until(() => {
      const audit = readNdjson(auditPath);
      const session = audit.find(row => row.kind === 'handshake_accepted' && row.role === 'bot');
      return session && audit.some(row => row.kind === 'baseline_acked' && row.sessionId === session.sessionId);
    }, 'real C# Bot baseline acknowledgement', contract.limits.scenarioTimeoutMs, guard);
    peer = openBrowserPeer(url, contract);
    const delta = await peer.exchange(guard);
    await until(() => pendingJson(resultPath), 'C# Bot result', contract.limits.scenarioTimeoutMs, guard);
    // Do not kill successful processes. The Bot result is emitted before orderly DS shutdown.
    await shutdown(server);
    await waitExit(bot);
    await until(() => peer.state.closed, 'browser peer close handshake', 10000);
    assert.ok([1000, 1001].includes(peer.state.closeCode), `Unexpected WebSocket close: ${peer.state.closeCode}`);
    const projection = verifyHelloEvidence(contract, json(resultPath), delta, readNdjson(auditPath));
    writeFileSync(join(dir, 'hello-projection.json'), JSON.stringify(projection, null, 2) + '\n');
    return projection;
  } finally {
    process.removeListener('SIGINT', interrupt);
    process.removeListener('SIGTERM', interrupt);
    peer?.close();
    await forceCleanup(bot);
    await forceCleanup(server);
  }
}

export async function runDevelopment({ root, verify = false, keepRunning = false, voxelRoot, nativeCoreRoot }) {
  if (verify && keepRunning) throw new Error('--verify and --keep-running are mutually exclusive.');
  root = realpathSync(root);
  mkdirSync(join(root, '.run/verification'), { recursive: true });
  const evidence = mkdtempSync(join(root, '.run/verification/run-'));
  console.log(`EVIDENCE_PATH=${evidence}`);
  const report = { version: 1, status: 'RUNNING', scope: verify ? 'native-and-hello-integration' : 'host-loading', evidence };
  const reportPath = join(evidence, 'verification.json');
  writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n');
  try {
    const roots = {
      LumioNativeCore: realpathSync(nativeCoreRoot || process.env.NATIVE_CORE_ROOT || join(root, '../LumioNativeCore')),
      LumioVoxelEngine: realpathSync(voxelRoot || process.env.VOXEL_ROOT || join(root, '../LumioVoxelEngine')),
      LumioGameRuntime: realpathSync(process.env.LumioRuntimeRoot || join(root, '../LumioGameRuntime')),
      LumioServer: realpathSync(process.env.LumioServerRoot || join(root, '../LumioServer')),
      LumioClient: realpathSync(process.env.LumioClientRoot || join(root, '../LumioClient')),
    };
    report.sources = sources(roots, root);
    command(process.execPath, [join(root, 'eng/generate-abi.mjs')], { cwd: root, log: join(evidence, 'generate.log') });
    const native = buildNative({ root, nativeCoreRoot: roots.LumioNativeCore, voxelRoot: roots.LumioVoxelEngine });
    report.native = native;
    report.dotnet = command('dotnet', ['--info'], { cwd: root, log: join(evidence, 'dotnet.log') });
    const serverOutput = command('cargo', ['build', '--locked', '--manifest-path', join(roots.LumioServer, 'Cargo.toml'),
      '--bin', 'lumio-server', '--features', 'test-harness', '--message-format=json-render-diagnostics', '--target-dir', join(root, '.build/server-target')],
    { cwd: roots.LumioServer, log: join(evidence, 'server-build.log') });
    const artifacts = serverOutput.split('\n').filter(line => line.startsWith('{')).map(line => JSON.parse(line));
    const serverExe = artifacts.findLast(row => row.reason === 'compiler-artifact' && row.target?.name === 'lumio-server' && row.executable)?.executable;
    if (!serverExe) throw new Error('Cargo did not identify the built lumio-server executable.');
    const managedEnv = { ...process.env, LumioArchRoot: root, LUMIO_ARCH_ROOT: root,
      LUMIO_ENGINE_SDK_PROJECT: join(root, 'engine/managed/Lumio.Engine.SDK/Lumio.Engine.SDK.csproj'),
      LumioRuntimeRoot: roots.LumioGameRuntime };
    // Always call incremental builds, even when older binaries already exist.
    for (const [index, step] of managedBuildPlan(roots, evidence, verify).entries()) {
      command('dotnet', step.args, { cwd: root, env: managedEnv, log: join(evidence, `managed-build-${index}.log`) });
    }
    const runtimeDll = join(evidence, 'runtime/Lumio.GameRuntime.HelloEntry.dll');
    const runtimeConfig = join(evidence, 'runtime/Lumio.GameRuntime.HelloEntry.runtimeconfig.json');
    const foundationDll = join(evidence, 'foundation/Lumio.Client.Bot.Host.dll');
    const helloDll = join(evidence, 'hello-bot/Lumio.Client.HelloBot.dll');
    const runtimeFiles = [runtimeDll, runtimeConfig, foundationDll, serverExe, ...(verify ? [helloDll] : [])];
    report.artifacts = Object.fromEntries(runtimeFiles.map(path => [path, hashFile(path)]));
    const proofOutput = command('dotnet', [foundationDll, 'foundation', '--engine-native', native.nativePath],
      { cwd: root, log: join(evidence, 'foundation.log') });
    const proofLine = proofOutput.split(/\r?\n/).find(line => line.startsWith('ENGINE_NATIVE '));
    const proof = parseNativeProof(proofLine ?? '');
    for (const key of ['buildId', 'abiHash', 'binarySha256']) assert.equal(proof[key], native[key], `Client ${key} proof mismatch.`);
    assert.equal(realpathSync(proof.nativePath), realpathSync(native.nativePath));
    assert.equal(hashFile(native.nativePath), native.binarySha256);
    console.log(`CLIENT ${proofLine}`);
    if (verify) {
      const env = { ...process.env, LUMIO_BUILD_ID: native.buildId, LUMIO_ABI_HASH: native.abiHash,
        LUMIO_NATIVE_TEST_PATH: native.nativePath, LUMIO_NATIVE_TEST_BUILD_ID: native.buildId, LUMIO_NATIVE_TEST_ABI_HASH: native.abiHash };
      command('cargo', ['test', '--locked', '--manifest-path', native.nativeManifest, '-p', 'lumio-engine-native',
        '--target', native.buildInputs.target, '--target-dir', native.targetDir], { cwd: root, env, log: join(evidence, 'native-tests.log') });
      const integrationProject = join(root, 'engine/managed/Lumio.Engine.NativeLoader.IntegrationTests/Lumio.Engine.NativeLoader.IntegrationTests.csproj');
      command('dotnet', ['test', integrationProject, '--configuration', 'Debug', '--logger', 'trx', '--results-directory', join(evidence, 'test-results')],
        { cwd: root, env, log: join(evidence, 'native-interop-tests.log') });
    }
    const contractPath = join(root, 'engine/wire/hello-wire-v1.json');
    const context = { root, evidence, native, serverExe, hostfxr: hostFxr(root), contractPath, contract: json(contractPath), runtimeDll, runtimeConfig, helloDll };
    const first = await round(context, 1, verify, keepRunning);
    if (verify) {
      const second = await round(context, 2, true, false);
      assert.deepEqual(second, first, 'Fresh-process Hello trace replay differs; compare round-1 and round-2 projections.');
      report.helloReplay = { rounds: 2, projection: first, scope: 'hello-wire-trace-only; not full ECS/GAS world determinism' };
    }
    report.status = 'PASS';
  } catch (error) {
    report.status = error.code === 'ENOENT' || error.message.startsWith('BLOCKED_ENV:') ? 'BLOCKED_ENV' : 'FAIL';
    report.error = error.message;
    throw error;
  } finally {
    writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n');
    console.log(`VERIFICATION_STATUS=${report.status}\nEVIDENCE_PATH=${evidence}`);
  }
}

async function main() {
  const options = { root: resolve(fileURLToPath(new URL('..', import.meta.url))) };
  const args = process.argv.slice(2);
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--verify') options.verify = true;
    else if (args[i] === '--keep-running') options.keepRunning = true;
    else if (['--voxel-root', '--native-core-root'].includes(args[i]) && args[i + 1]) options[args[i++] === '--voxel-root' ? 'voxelRoot' : 'nativeCoreRoot'] = args[i];
    else throw new Error(`Unknown or incomplete option: ${args[i]}`);
  }
  await runDevelopment(options);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(`${error.code === 'ENOENT' ? 'BLOCKED_ENV' : 'FAIL'}: ${error.stack ?? error.message}`); process.exitCode = error.code === 'ENOENT' ? 2 : 1; });
}
