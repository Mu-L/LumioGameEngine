import { spawn, spawnSync } from 'node:child_process';
import { appendFileSync, writeFileSync } from 'node:fs';

export function command(executable, args, { cwd, env = process.env, log } = {}) {
  const result = spawnSync(executable, args, { cwd, env, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  if (log) writeFileSync(log, `$ ${JSON.stringify([executable, ...args])}\n${result.stdout ?? ''}${result.stderr ?? ''}`);
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${executable} exited ${result.status ?? result.signal}. ${result.stdout ?? ''}`);
  return result.stdout;
}

export function startLogged(executable, args, { cwd, env = process.env, log }) {
  writeFileSync(log, `$ ${JSON.stringify([executable, ...args])}\n`);
  const child = spawn(executable, args, { cwd, env, stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true });
  const state = { child, stdout: '', closed: false, code: null, signal: null, error: null };
  child.stdin.on('error', () => {}); // An exited child is handled through its exit/error state.
  child.stdout.on('data', bytes => {
    state.stdout = (state.stdout + bytes.toString()).slice(-1024 * 1024);
    appendFileSync(log, bytes);
  });
  child.stderr.on('data', bytes => appendFileSync(log, bytes));
  child.on('error', error => { state.error = error; });
  state.done = new Promise(resolve => child.once('close', (code, signal) => {
    Object.assign(state, { closed: true, code, signal });
    resolve(state);
  }));
  return state;
}

export function assertAlive(state) {
  if (state.error) throw state.error;
  if (state.closed) throw new Error(`Process ${state.child.pid} exited before proof completed: code=${state.code}, signal=${state.signal}`);
}

export function waitExit(state, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`Process ${state.child.pid} did not exit within ${timeoutMs}ms.`)), timeoutMs);
    state.done.then(result => {
      clearTimeout(timer);
      if (result.error) reject(result.error);
      else if (result.code !== 0) reject(new Error(`Process ${result.child.pid} exited ${result.code ?? result.signal}.`));
      else resolve(result);
    });
  });
}

export async function forceCleanup(state) {
  if (!state || state.closed) return;
  state.child.kill('SIGKILL');
  // Always reap the child, but a forced kill is never accepted as passing evidence.
  await Promise.race([state.done, new Promise(resolve => { const timer = setTimeout(resolve, 5000); timer.unref(); })]);
}
