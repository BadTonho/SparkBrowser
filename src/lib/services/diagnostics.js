/** Bounded, in-memory counters. Call arguments and error messages are deliberately excluded.
 * @param {number} capacity
 */
export function createDiagnostics(capacity = 256) {
  /** @type {{ command: string, duration_ms: number, success: boolean }[]} */
  const samples = [];
  /** @type {Map<string, { count: number, errors: number, duration_ms: number }>} */
  const totals = new Map();
  return {
    /** @param {string} name @param {number} duration @param {boolean} success */
    record(name, duration, success) {
      const command = /^[a-z_]{1,64}$/.test(name) ? name : 'invalid_command';
      if (!Number.isFinite(duration) || duration < 0) return;
      if (!totals.has(command) && totals.size >= 64) return;
      const total = totals.get(command) ?? { count: 0, errors: 0, duration_ms: 0 };
      total.count++; total.errors += Number(!success); total.duration_ms += duration;
      totals.set(command, total);
      samples.push({ command, duration_ms: duration, success });
      if (samples.length > capacity) samples.shift();
    },
    snapshot() {
      return [...totals].map(([command, total]) => {
        const recent = samples.filter(sample => sample.command === command).map(sample => sample.duration_ms).sort((a, b) => a - b);
        return { command, count: total.count, errors: total.errors, mean_ms: total.duration_ms / total.count,
          p95_recent_ms: recent[Math.max(0, Math.ceil(recent.length * 0.95) - 1)] ?? 0, recent_samples: recent.length };
      });
    },
  };
}
export const ipcDiagnostics = createDiagnostics();
