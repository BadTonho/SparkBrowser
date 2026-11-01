import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createDiagnostics } from '../src/lib/services/diagnostics.js';

test('diagnostics keep cumulative counts with bounded recent timings and exclude URL-shaped labels', () => {
  const metrics = createDiagnostics(4);
  for (let i = 1; i <= 10; i++) metrics.record('switch_tab', i, i !== 3);
  assert.deepEqual(metrics.snapshot(), [{ command: 'switch_tab', count: 10, errors: 1, mean_ms: 5.5, p95_recent_ms: 10, recent_samples: 4 }]);
  metrics.record('https://private.example/token', 1, true);
  assert.ok(!JSON.stringify(metrics.snapshot()).includes('private.example'));
  metrics.record('switch_tab', NaN, true);
  assert.equal(metrics.snapshot()[0].count, 10);
});
