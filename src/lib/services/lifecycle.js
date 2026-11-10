/**
 * @typedef {{active: () => boolean, guard: <T extends any[]>(callback: (...args: T) => any) => (...args: T) => any, own: (cleanup: () => void) => void}} LifecycleScope
 */
/** One resource set per domain, shared by mounted consumers and released once.
 * @param {(scope: LifecycleScope) => void} start
 */
export function createSharedLifecycle(start) {
  /** @type {{users: number, alive: boolean, stop: () => void} | undefined} */
  let current;
  return () => {
    if (!current) {
      /** @type {(() => void)[]} */
      const cleanups = [];
      const run = { users: 0, alive: true, stop: () => {} };
      /** @type {LifecycleScope} */
      const scope = {
        active: () => run.alive,
        guard: callback => (...args) => { if (run.alive) return callback(...args); },
        own(cleanup) {
          if (run.alive) cleanups.push(cleanup);
          else cleanup();
        },
      };
      run.stop = () => {
        run.alive = false;
        const errors = [];
        for (const cleanup of cleanups.splice(0).reverse()) {
          try { cleanup(); } catch (error) { errors.push(error); }
        }
        if (errors.length) throw new AggregateError(errors, 'Domain cleanup failed');
      };
      current = run;
      try { start(scope); } catch (error) { current = undefined; run.stop(); throw error; }
    }
    const run = current;
    run.users++;
    let released = false;
    return () => {
      if (released) return;
      released = true;
      if (--run.users === 0) {
        current = undefined;
        run.stop();
      }
    };
  };
}
