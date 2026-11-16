import { test } from 'node:test';
import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { stopDriverProcess } from './native/driver-cleanup.mjs';

function driver() {
  return Object.assign(new EventEmitter(), { pid: 123, exitCode: null, signalCode: null, detached: false,
    unref() { this.detached = true; }, stderr: { destroy() {} }, stdout: { destroy() {} } });
}

test('native cleanup targets only the scenario tree and waits for its exit', async () => {
  const child = driver(), killer = new EventEmitter();
  const stopping = stopDriverProcess(child, { platform: 'win32', spawnProcess(command, args) {
    assert.equal(command, 'taskkill');
    assert.deepEqual(args, ['/PID', '123', '/T', '/F']);
    return killer;
  } });
  killer.emit('close', 0);
  child.emit('exit', 0);
  await stopping;
  assert.equal(child.detached, false);
});

test('denied termination and missing tools fail cleanup instead of waiting forever', async () => {
  for (const event of ['denied', 'missing']) {
    const child = driver(), killer = new EventEmitter();
    const stopping = stopDriverProcess(child, { platform: 'win32', spawnProcess: () => killer });
    if (event === 'denied') killer.emit('close', 1);
    else killer.emit('error', new Error('missing taskkill'));
    await assert.rejects(stopping, /permissions|missing taskkill/);
    assert.equal(child.detached, true);
    assert.equal(child.listenerCount('exit'), 0);
  }
});

test('a driver that never exits has bounded cleanup and Linux targets its process group', async () => {
  const child = driver();
  await assert.rejects(stopDriverProcess(child, { platform: 'linux', timeoutMs: 10, killGroup(pid, signal) {
    assert.equal(pid, -123);
    assert.equal(signal, 'SIGKILL');
  } }), /Timed out/);
  child.exitCode = 0;
  await stopDriverProcess(child, { spawnProcess() { assert.fail('already exited'); } });
});
