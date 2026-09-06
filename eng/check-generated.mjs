import { cpSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { command } from './process-tools.mjs';

// Generate in isolation: validation must never rewrite a developer's source tree.
const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const scratch = mkdtempSync(join(tmpdir(), 'lumio-generated-check-'));
const outputs = [
  'engine/native/modules/sdk-native/src/abi_generated.rs',
  'engine/native/modules/sdk-native/include/lumio_engine.h',
  'engine/managed/Lumio.Engine.NativeLoader/AbiConstants.g.cs',
];
try {
  for (const path of ['eng', 'engine/abi', 'engine/wire']) cpSync(join(root, path), join(scratch, path), { recursive: true });
  command(process.execPath, [join(scratch, 'eng/generate-abi.mjs')], { cwd: scratch });
  const stale = outputs.filter(path => !readFileSync(join(root, path)).equals(readFileSync(join(scratch, path))));
  if (stale.length) throw new Error(`Generated bindings are stale: ${stale.join(', ')}. Run node eng/generate-abi.mjs, review the generated diff, then rerun tests.`);
  console.log('PASS: checked-in ABI bindings equal generator output.');
} catch (error) {
  console.error(`FAIL: ${error.message}`);
  process.exitCode = 1;
} finally { rmSync(scratch, { recursive: true, force: true }); }
