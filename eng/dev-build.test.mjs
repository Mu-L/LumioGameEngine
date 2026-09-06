import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { sourceFingerprint, buildIdentity, rewriteDependencyRoots } from './dev-build.mjs';
import { command, startLogged, assertAlive, waitExit } from './process-tools.mjs';

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'lumio inputs with spaces-'));
  for (const dir of ['engine/engine/native', 'engine/engine/abi', 'core/crates/demo', 'voxel/crates/demo']) mkdirSync(join(root, dir), { recursive: true });
  writeFileSync(join(root, 'engine/engine/abi/native-abi.json'), '{}');
  writeFileSync(join(root, 'core/crates/demo/lib.rs'), 'pub fn value() -> u32 { 1 }');
  return { root, roots: { engine: join(root, 'engine'), nativeCore: join(root, 'core'), voxel: join(root, 'voxel') } };
}
test('moving the same inputs between worktree paths does not change source identity', () => {
  const a = fixture(), b = fixture();
  try { assert.equal(sourceFingerprint(a.roots), sourceFingerprint(b.roots)); }
  finally { rmSync(a.root, { recursive: true }); rmSync(b.root, { recursive: true }); }
});
test('source edits and untracked build inputs invalidate identity, external documentation does not', () => {
  const f = fixture();
  try {
    const before = sourceFingerprint(f.roots);
    mkdirSync(join(f.roots.engine, '.spec'));
    writeFileSync(join(f.roots.engine, '.spec/review.md'), 'a new review');
    assert.equal(sourceFingerprint(f.roots), before);
    writeFileSync(join(f.roots.nativeCore, 'crates/demo/new.rs'), 'pub fn new_value() {}');
    assert.notEqual(sourceFingerprint(f.roots), before);
  } finally { rmSync(f.root, { recursive: true }); }
});
test('configuration, target and toolchain are part of BuildId', () => {
  const base = { configuration: 'debug', target: 'x86_64-unknown-linux-gnu', rustc: 'test rustc', flags: {} };
  for (const change of [{ configuration: 'release' }, { target: 'x86_64-pc-windows-msvc' }, { rustc: 'other rustc' }, { flags: { RUSTFLAGS: '-C opt-level=2' } }]) {
    assert.notEqual(buildIdentity('source', base), buildIdentity('source', { ...base, ...change }));
  }
});
test('explicit dependency roots preserve extra manifest attributes and internal paths', () => {
  const input = 'lumio-kernel = { path = "../../LumioNativeCore/crates/lumio-kernel", features = ["x"] }\nlocal = { path = "../clr-host" }\nvoxel = { path = "../../LumioVoxelEngine/crates/lumio-voxel-world" }';
  const output = rewriteDependencyRoots(input, { LumioNativeCore: '/work/core path', LumioVoxelEngine: '/work/voxel path' });
  assert.ok(output.includes('path = "/work/core path/crates/lumio-kernel", features = ["x"]'));
  assert.ok(output.includes('path = "../clr-host"'));
  assert.ok(output.includes('path = "/work/voxel path/crates/lumio-voxel-world"'));
});
test('commands preserve argument boundaries without invoking a shell', () => {
  assert.equal(command(process.execPath, ['-e', 'process.stdout.write(JSON.stringify(process.argv.slice(1)))', 'path with spaces', 'x;y']), '["path with spaces","x;y"]');
  assert.throws(() => command(process.execPath, ['-e', 'process.exit(7)']), /exited 7/);
});
test('an early zero exit is not readiness, but a completed command may exit zero', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'lumio-process-'));
  try {
    const child = startLogged(process.execPath, ['-e', 'process.exit(0)'], { log: join(dir, 'child.log') });
    await waitExit(child);
    assert.throws(() => assertAlive(child), /before proof/);
  } finally { rmSync(dir, { recursive: true }); }
});
