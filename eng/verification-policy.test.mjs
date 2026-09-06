import assert from 'node:assert/strict';
import { test } from 'node:test';
import { selectChecks, validateLock, assertRequiredResults } from './verification-policy.mjs';

test('documentation does not require a native SDK or other repositories', () => {
  assert.deepEqual(selectChecks(['README.md', '.spec/AGENTS.md']).checks, { tools: true, managed: false, integration: false });
});
test('managed unit-test-only edits select managed tests', () => {
  assert.deepEqual(selectChecks(['engine/managed/Lumio.Engine.NativeLoader.Tests/IdentityTests.cs']).checks,
    { tools: true, managed: true, integration: false });
});
for (const path of ['engine/abi/native-abi.json', 'engine/wire/hello-wire-v1.json', 'engine/native/Cargo.toml',
  'engine/managed/Lumio.Engine.NativeLoader/NativeEngineLoader.cs', 'eng/dev-run.mjs',
  '.github/workflows/repository-policy.yml', 'eng/workspace-lock.json', 'new-unclassified-component/file.cs']) {
  test(`runtime, build and unknown paths fail closed: ${path}`, () => assert.equal(selectChecks([path]).checks.integration, true));
}
test('a removed source path is still classified as runtime-affecting', () => {
  assert.equal(selectChecks(['engine/native/modules/removed.rs']).checks.integration, true);
});
test('unsafe and non-relative paths are rejected', () => {
  for (const path of ['/etc/passwd', '../outside', 'a\\b', '']) assert.throws(() => selectChecks([path]));
});
const plan = selectChecks(['engine/abi/native-abi.json']);
const good = { plan: { result: 'success' }, tools: { result: 'success' }, managed: { result: 'success' }, integration: { result: 'success' } };
test('gate accepts complete positive evidence', () => assert.doesNotThrow(() => assertRequiredResults(plan, good)));
for (const result of ['failure', 'cancelled', 'skipped', undefined]) {
  test(`gate rejects a required integration lane with ${result}`, () => {
    assert.throws(() => assertRequiredResults(plan, { ...good, integration: { result } }));
  });
}
test('gate accepts an explicitly out-of-scope lane, not failed planning', () => {
  const docs = selectChecks(['README.md']);
  assert.doesNotThrow(() => assertRequiredResults(docs, { ...good, managed: { result: 'skipped' }, integration: { result: 'skipped' } }));
  assert.throws(() => assertRequiredResults(docs, { ...good, plan: { result: 'failure' } }));
});
test('gate rejects incomplete or weakened selection', () => {
  for (const checks of [{}, { tools: false, managed: false, integration: false }, { tools: true, managed: 'false', integration: false }]) {
    assert.throws(() => assertRequiredResults({ version: 1, checks }, good));
  }
});
test('lock validation rejects moving, short, missing and unexpected dependencies', () => {
  const repositories = Object.fromEntries(['LumioNativeCore', 'LumioVoxelEngine', 'LumioGameRuntime', 'LumioServer', 'LumioClient'].map(name => [name, 'a'.repeat(40)]));
  assert.doesNotThrow(() => validateLock({ version: 1, repositories }));
  for (const ref of ['main', 'abcdef0', '', '../main']) assert.throws(() => validateLock({ version: 1, repositories: { ...repositories, LumioServer: ref } }));
  assert.throws(() => validateLock({ version: 1, repositories: { ...repositories, Other: 'a'.repeat(40) } }));
});
