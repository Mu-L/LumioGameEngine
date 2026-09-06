import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { wireMessage, readNdjson, until, verifyHelloEvidence } from './hello-smoke.mjs';

const hash = createHash('sha256').update('Hello World').digest('hex');
const contract = {
  messages: { Handshake: { required: { messageType: 'const:Handshake', contractId: 'const:lumio.hello-wire.v1', role: 'enum:roles' } } },
  hash: { example: { payload: 'Hello World' } },
  process: { auditEventKinds: {
    tick_committed: { required: ['tickId', 'revision', 'senders'] },
    ingress_received: { required: ['sender', 'sequence', 'payloadSha256'] },
    delta_routed: { required: ['sender', 'sequence', 'tickId', 'revision', 'payloadSha256'] },
    server_shutdown: { required: [] },
  } },
};
function evidence() {
  const first = { sender: 'browser', sequence: 1, tickId: 1, revision: 1, payloadSha256: hash };
  const second = { sender: 'bot', sequence: 1, tickId: 2, revision: 2, payloadSha256: hash };
  return {
    bot: { ok: true, role: 'bot', received: [first], sent: { sequence: 1, payloadSha256: hash } },
    delta: { ...second, payload: 'Hello World', commandSequence: 1 },
    audit: [first, second].flatMap(row => [
      { kind: 'ingress_received', ...row },
      { kind: 'tick_committed', tickId: row.tickId, revision: row.revision, senders: [row.sender] },
      { kind: 'delta_routed', ...row },
    ]).concat({ kind: 'server_shutdown' }),
  };
}
test('test peer obtains constants and required fields from the production contract', () => {
  assert.deepEqual(wireMessage(contract, 'Handshake', { role: 'browser' }), { role: 'browser', messageType: 'Handshake', contractId: 'lumio.hello-wire.v1' });
  assert.throws(() => wireMessage(contract, 'Handshake', {}));
  assert.throws(() => wireMessage(contract, 'Handshake', { role: 'browser', unknown: 1 }));
  assert.throws(() => wireMessage(contract, 'Handshake', { role: 'browser', messageType: 'Other' }));
});
test('complete Hello receipts and commit audit are accepted', () => {
  const { bot, delta, audit } = evidence();
  assert.equal(verifyHelloEvidence(contract, bot, delta, audit).length, 2);
});
for (const mutation of ['echo', 'missing-commit', 'wrong-hash', 'duplicate', 'no-shutdown', 'failed-bot']) {
  test(`Hello evidence rejects ${mutation}`, () => {
    const e = evidence();
    if (mutation === 'echo') e.delta.tickId = 0;
    if (mutation === 'missing-commit') e.audit = e.audit.filter(row => row.kind !== 'tick_committed');
    if (mutation === 'wrong-hash') e.bot.received[0].payloadSha256 = '0'.repeat(64);
    if (mutation === 'duplicate') e.bot.received.push(e.bot.received[0]);
    if (mutation === 'no-shutdown') e.audit.pop();
    if (mutation === 'failed-bot') e.bot.ok = false;
    assert.throws(() => verifyHelloEvidence(contract, e.bot, e.delta, e.audit));
  });
}
test('NDJSON ignores only an in-progress trailing line, not corrupt completed evidence', () => {
  const dir = mkdtempSync(join(tmpdir(), 'lumio-ndjson-'));
  try {
    const file = join(dir, 'audit.ndjson');
    writeFileSync(file, '{"kind":"one"}\n{"kind":');
    assert.deepEqual(readNdjson(file), [{ kind: 'one' }]);
    writeFileSync(file, '{broken}\n');
    assert.throws(() => readNdjson(file));
  } finally { rmSync(dir, { recursive: true }); }
});
test('bounded waits cannot turn a timeout into success', async () => {
  await assert.rejects(until(() => false, 'missing proof', 5), /Timed out/);
});
