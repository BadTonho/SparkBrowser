import test from 'node:test';
import assert from 'node:assert/strict';
import { connect } from 'node:net';
import { createNetworkPrivacyFixture } from './native/network-privacy-fixture.mjs';

test('loopback fixtures record redirects and SOCKS hostnames without forwarding', async () => {
  const fixture = await createNetworkPrivacyFixture();
  try {
    const response = await fetch(`${fixture.origin}/redirect-http`);
    assert.equal(await response.text(), 'fixture');
    assert.deepEqual(fixture.httpRequests, ['/redirect-http', '/page']);
    const result = await new Promise((resolve, reject) => {
      const socket = connect(fixture.socksPort, '127.0.0.1');
      socket.on('error', reject);
      let step = 0;
      socket.on('connect', () => socket.write(Buffer.from([5, 1, 0])));
      socket.on('data', data => {
        if (step++ === 0) {
          assert.deepEqual([...data], [5, 0]);
          const host = Buffer.from('fixture.invalid');
          const port = Buffer.from([0x01, 0xbb]);
          socket.write(Buffer.concat([Buffer.from([5, 1, 0, 3, host.length]), host, port]));
        } else resolve([...data]);
      });
    });
    assert.equal(result[1], 1);
    assert.deepEqual(fixture.socksRequests, [{ command: 1, host: 'fixture.invalid', port: 443 }]);
  } finally { await fixture.close(); }
});
