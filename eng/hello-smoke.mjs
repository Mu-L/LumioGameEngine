// Test-only protocol peer. Production consumers remain in LumioClient/LumioServer.
// Message constants and required fields come from the existing wire contract.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { setTimeout as delay } from 'node:timers/promises';

export function wireMessage(contract, name, values = {}) {
  const shape = contract.messages?.[name];
  if (!shape) throw new Error(`Unknown contract message: ${name}`);
  const message = { ...values };
  for (const [key, type] of Object.entries(shape.required)) {
    if (type.startsWith('const:')) {
      const constant = type.slice(6);
      if (key in message && message[key] !== constant) throw new Error(`Wrong constant ${name}.${key}`);
      message[key] = constant;
    }
    if (!(key in message)) throw new Error(`Missing ${name}.${key}`);
  }
  for (const key of Object.keys(message)) {
    if (!(key in shape.required) && !(key in (shape.optional ?? {}))) throw new Error(`Unknown ${name}.${key}`);
  }
  return message;
}

export function readNdjson(path) {
  let text;
  try { text = readFileSync(path, 'utf8'); } catch (error) { if (error.code === 'ENOENT') return []; throw error; }
  // Only the final, incomplete line may be ignored while a producer is writing.
  return text.split('\n').slice(0, -1).filter(Boolean).map(line => JSON.parse(line));
}

export async function until(check, description, timeoutMs = 30000, guard = () => {}) {
  const deadline = performance.now() + timeoutMs;
  do {
    guard();
    const value = check();
    if (value) return value;
    await delay(25);
  } while (performance.now() < deadline);
  throw new Error(`Timed out: ${description}`);
}

export function openBrowserPeer(url, contract) {
  const ws = new WebSocket(url, contract.transport.subprotocol);
  const state = { messages: [], error: null, open: false, closed: false, closeCode: null };
  ws.addEventListener('open', () => { state.open = true; });
  ws.addEventListener('error', () => { state.error = new Error('Browser test peer WebSocket failed.'); });
  ws.addEventListener('close', event => { state.closed = true; state.closeCode = event.code; });
  ws.addEventListener('message', event => {
    try {
      const message = JSON.parse(String(event.data));
      if (message.messageType === 'Error') throw new Error(`Server rejected test input: ${message.code}: ${message.detail}`);
      if (!contract.messages[message.messageType]) throw new Error(`Unknown server message ${message.messageType}`);
      state.messages.push(message);
    } catch (error) { state.error = error; }
  });
  return {
    state,
    close: () => { if (!state.closed) ws.close(); },
    async exchange(guard) {
      const wait = (check, name) => until(check, name, contract.limits.scenarioTimeoutMs, () => {
        guard();
        if (state.error) throw state.error;
        if (state.closed) throw new Error('Browser test peer closed before the exchange completed.');
      });
      await wait(() => state.open, 'browser peer connection');
      ws.send(JSON.stringify(wireMessage(contract, 'Handshake', { role: 'browser', clientName: 'lumio-sdk-ci-peer' })));
      const ack = await wait(() => state.messages.find(m => m.messageType === 'HandshakeAck'), 'browser handshake');
      assert.equal(ack.accepted, true);
      assert.equal(ack.role, 'browser');
      assert.equal(ack.contractId, contract.contractId);
      const snapshot = await wait(() => state.messages.find(m => m.messageType === 'FullSnapshot'), 'browser baseline');
      assert.equal(snapshot.revision, 0);
      assert.equal(snapshot.tickId, 0);
      assert.deepEqual(snapshot.helloLog, []);
      ws.send(JSON.stringify(wireMessage(contract, 'BaselineAck', { revision: snapshot.revision })));
      const payload = contract.hash.example.payload;
      const payloadSha256 = createHash('sha256').update(payload, 'utf8').digest('hex');
      assert.equal(payloadSha256, contract.hash.example.payloadSha256);
      ws.send(JSON.stringify(wireMessage(contract, 'InputCommand', {
        sender: 'browser', sequence: 1, payload, payloadSha256, sentAtMs: Date.now(),
      })));
      return wait(() => state.messages.find(m => m.messageType === 'Delta'), 'authoritative bot-to-browser Delta');
    },
  };
}

export function verifyHelloEvidence(contract, bot, browserDelta, audit) {
  assert.equal(bot.ok, true, bot.reason ?? 'Bot result is not successful.');
  assert.equal(bot.role, 'bot');
  assert.equal(bot.received.length, 1, 'Bot must receive exactly one authoritative browser Delta.');
  const hash = createHash('sha256').update(contract.hash.example.payload, 'utf8').digest('hex');
  assert.equal(bot.sent.sequence, 1);
  assert.equal(bot.sent.payloadSha256, hash);
  assert.equal(browserDelta.payload, contract.hash.example.payload);
  assert.equal(browserDelta.commandSequence, 1);
  const fields = ['sender', 'sequence', 'tickId', 'revision', 'payloadSha256'];
  const project = row => Object.fromEntries(fields.map(key => [key, row[key]]));
  const expected = [
    { sender: 'browser', sequence: 1, tickId: 1, revision: 1, payloadSha256: hash },
    { sender: 'bot', sequence: 1, tickId: 2, revision: 2, payloadSha256: hash },
  ];
  assert.deepEqual([project(bot.received[0]), project(browserDelta)], expected);
  for (const row of audit) {
    const rule = contract.process.auditEventKinds[row.kind];
    assert.ok(rule, `Undeclared audit event: ${row.kind}`);
    for (const field of rule.required) assert.ok(field in row, `Missing ${row.kind}.${field}`);
    assert.ok(!['ingress_rejected', 'handshake_rejected'].includes(row.kind), `Unexpected rejection: ${JSON.stringify(row)}`);
  }
  const commits = audit.filter(row => row.kind === 'tick_committed');
  assert.equal(commits.length, 2, 'There must be two real authoritative commits, not a network echo.');
  expected.forEach((row, i) => {
    assert.equal(commits[i].tickId, row.tickId);
    assert.equal(commits[i].revision, row.revision);
    assert.deepEqual(commits[i].senders, [row.sender]);
  });
  const ingress = audit.filter(row => row.kind === 'ingress_received');
  assert.deepEqual(ingress.map(row => [row.sender, row.sequence, row.payloadSha256]), expected.map(row => [row.sender, row.sequence, row.payloadSha256]));
  assert.deepEqual(audit.filter(row => row.kind === 'delta_routed').map(project), expected);
  assert.equal(audit.filter(row => row.kind === 'server_shutdown').length, 1, 'Graceful server shutdown must be audited.');
  return expected; // Deliberately excludes wall-clock timing; this is a Hello trace projection, NOT a full-world hash.
}
