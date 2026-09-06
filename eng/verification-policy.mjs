import { appendFileSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repositories = ['LumioNativeCore', 'LumioVoxelEngine', 'LumioGameRuntime', 'LumioServer', 'LumioClient'];
const outputNames = ['native_core', 'voxel', 'runtime', 'server', 'client'];

/** Conservative selection: only known non-runtime paths may omit integration. */
export function selectChecks(paths) {
  const checks = { tools: true, managed: false, integration: false };
  for (const path of paths) {
    if (typeof path !== 'string' || !path || path.startsWith('/') || path.includes('\\') || path.split('/').includes('..')) {
      throw new Error(`Invalid changed path: ${path}`);
    }
    if (/^engine\/managed\/[^/]+\.Tests\//.test(path)) {
      checks.managed = true;
      continue;
    }
    if (path.startsWith('.spec/') || /^(README(?:\.en)?\.md|LICENSE|\.gitattributes|\.gitignore)$/.test(path)
        || (path.endsWith('.md') && !path.startsWith('.github/'))) continue;
    checks.managed = true;
    checks.integration = true;
  }
  return { version: 1, checks, paths };
}

export function validateLock(lock) {
  if (lock?.version !== 1 || !lock.repositories || Object.keys(lock.repositories).length !== repositories.length) {
    throw new Error('workspace-lock.json must contain exactly the five SDK dependency repositories.');
  }
  for (const name of repositories) {
    if (!/^[0-9a-f]{40}$/.test(lock.repositories[name] ?? '')) {
      throw new Error(`${name} must be pinned to a full lowercase commit SHA, never a moving branch.`);
    }
  }
  return lock;
}

/** A required but skipped/cancelled/missing check is NOT success. */
export function assertRequiredResults(plan, needs) {
  if (plan?.version !== 1 || needs?.plan?.result !== 'success') throw new Error('Verification planning did not succeed.');
  const names = ['tools', 'managed', 'integration'];
  if (!plan.checks || Object.keys(plan.checks).sort().join(',') !== [...names].sort().join(',')) {
    throw new Error('Missing or unknown verification lanes.');
  }
  if (plan.checks.tools !== true) throw new Error('Tooling checks are always required.');
  for (const name of names) {
    if (typeof plan.checks[name] !== 'boolean') throw new Error(`Invalid selection for ${name}.`);
    const result = needs[name]?.result;
    if (plan.checks[name] ? result !== 'success' : !['success', 'skipped'].includes(result)) {
      throw new Error(`${name}: required=${plan.checks[name]}, result=${result ?? 'missing'}`);
    }
  }
}

function main() {
  if (process.argv[2] === 'gate') {
    assertRequiredResults(JSON.parse(process.env.LUMIO_PLAN ?? 'null'), JSON.parse(process.env.LUMIO_NEEDS ?? 'null'));
    console.log('PASS: every selected verification lane completed successfully.');
    return;
  }
  if (process.argv[2] !== 'plan') throw new Error('usage: node eng/verification-policy.mjs plan|gate');
  const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
  const lock = validateLock(JSON.parse(readFileSync(resolve(root, 'eng/workspace-lock.json'), 'utf8')));
  const base = process.env.LUMIO_BASE;
  const head = process.env.LUMIO_HEAD;
  let paths = ['unknown-change']; // A missing base, initial push or failed diff must run everything.
  if (/^[0-9a-f]{40}$/.test(base ?? '') && !/^0+$/.test(base) && /^[0-9a-f]{40}$/.test(head ?? '')) {
    const diff = spawnSync('git', ['diff', '--no-renames', '--name-only', '-z', base, head, '--'], { cwd: root, encoding: 'utf8' });
    if (diff.status === 0) paths = diff.stdout.split('\0').filter(Boolean);
  }
  const plan = selectChecks(paths);
  console.log(JSON.stringify(plan, null, 2));
  if (process.env.GITHUB_OUTPUT) {
    const outputs = { plan: JSON.stringify(plan), ...plan.checks };
    repositories.forEach((name, i) => { outputs[outputNames[i]] = lock.repositories[name]; });
    appendFileSync(process.env.GITHUB_OUTPUT, Object.entries(outputs).map(([key, value]) => `${key}=${value}\n`).join(''));
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); } catch (error) { console.error(`FAIL: ${error.message}`); process.exitCode = 1; }
}
