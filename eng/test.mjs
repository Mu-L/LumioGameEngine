import { readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { command } from './process-tools.mjs';
import { runDevelopment } from './dev-run.mjs';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const mode = process.argv[2] ?? 'tools';
async function main() {
  if (!['tools', 'managed', 'integration', 'all'].includes(mode) || process.argv.length > 3) {
    throw new Error('usage: node eng/test.mjs tools|managed|integration|all');
  }
  const run = (exe, args) => { const output = command(exe, args, { cwd: root }); if (output) process.stdout.write(output); };
  if (mode === 'tools' || mode === 'all') {
    const tests = readdirSync(join(root, 'eng')).filter(name => name.endsWith('.test.mjs')).map(name => join(root, 'eng', name));
    run(process.execPath, ['--test', ...tests, 'eng/verify-wire.mjs', 'eng/verify-hello-wire.mjs']);
    run(process.execPath, ['eng/verify-wire.mjs']);
    run(process.execPath, ['eng/verify-hello-wire.mjs']);
    run(process.execPath, ['eng/check-generated.mjs']);
    run(process.execPath, ['.spec/tools/spec-lint.mjs']);
  }
  if (mode === 'managed' || mode === 'all') {
    run('dotnet', ['test', 'engine/managed/Lumio.Engine.NativeLoader.Tests/Lumio.Engine.NativeLoader.Tests.csproj',
      '--configuration', 'Debug', '--logger', 'trx', '--results-directory', '.run/managed-test-results']);
  }
  if (mode === 'integration' || mode === 'all') await runDevelopment({ root, verify: true });
}
main().catch(error => { console.error(`${error.code === 'ENOENT' ? 'BLOCKED_ENV' : 'FAIL'}: ${error.message}`); process.exitCode = error.code === 'ENOENT' ? 2 : 1; });
