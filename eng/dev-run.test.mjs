import assert from 'node:assert/strict';
import { test } from 'node:test';
import { managedBuildPlan, parseNativeProof } from './dev-run.mjs';

test('managed build plan always builds Runtime and Foundation before selecting outputs', () => {
  const roots = { LumioGameRuntime: '/workspace/runtime with spaces', LumioClient: '/workspace/client with spaces' };
  const plan = managedBuildPlan(roots, '/evidence/fresh-run', true);
  assert.equal(plan.length, 3);
  for (const step of plan) {
    assert.equal(step.args[0], 'build');
    assert.ok(!step.args.includes('--no-build'));
    assert.ok(step.output.includes('fresh-run'));
    assert.ok(step.project.includes('with spaces'));
  }
  assert.equal(managedBuildPlan(roots, '/other-run', false).length, 2);
});
test('native proof parses paths with spaces and requires all three identities', () => {
  const line = `ENGINE_NATIVE path=C:\\path with spaces\\engine.dll buildId=${'a'.repeat(32)} abiHash=${'b'.repeat(64)} binarySha256=${'c'.repeat(64)}`;
  const proof = parseNativeProof(line);
  assert.equal(proof.nativePath, 'C:\\path with spaces\\engine.dll');
  assert.equal(proof.binarySha256, 'c'.repeat(64));
  assert.throws(() => parseNativeProof(line.replace(/ binarySha256=.*/, '')));
  assert.throws(() => parseNativeProof('SERVER_READY {}'));
});
