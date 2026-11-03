/** Serialize writes across origins while retaining only each origin's latest pending value.
 * @param {(origin: string, factor: number) => Promise<any>} send
 * @param {(snapshot: any, origin: string) => void} apply
 * @param {() => Promise<any>} recover
 * @param {(error: unknown, origin: string) => void} failed
 * @param {(origin: string, pending: boolean) => void} changed
 */
export function createZoomWriter(send, apply, recover, failed, changed) {
  /** @type {Map<string, number>} */
  const pending = new Map();
  let running = false;
  /** @type {(() => void)[]} */
  let idle = [];
  async function drain() {
    try {
      while (pending.size) {
        const entry = pending.entries().next().value;
        if (!entry) break;
        const [origin, factor] = entry;
        pending.delete(origin);
        try { apply(await send(origin, factor), origin); }
        catch (error) {
          try { apply(await recover(), origin); } catch { /* Preserve the last known snapshot. */ }
          failed(error, origin);
        }
        changed(origin, pending.has(origin));
      }
    } finally {
      running = false;
      const waiting = idle; idle = []; waiting.forEach(resolve => resolve());
    }
  }
  return {
    /** @param {string} origin @param {number} factor */
    schedule(origin, factor) {
      pending.set(origin, factor);
      changed(origin, true);
      if (!running) { running = true; queueMicrotask(() => { void drain(); }); }
    },
    async flush() { if (running) await new Promise(resolve => idle.push(() => resolve(undefined))); },
  };
}
