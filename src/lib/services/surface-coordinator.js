/** Mutual exclusion without imports between feature stores. */
export function createSurfaceCoordinator() {
  /** @type {Map<string, () => Promise<void>>} */
  const closers = new Map();
  let revision = 0;
  return {
    /** @param {string} name @param {() => Promise<void>} close */
    register(name, close) { closers.set(name, close); },
    /** @param {string} name */
    async claim(name) {
      const request = ++revision;
      for (const [other, close] of closers) {
        if (request !== revision) break;
        if (other !== name) await close();
      }
      return () => request === revision;
    },
  };
}
export const surfaceCoordinator = createSurfaceCoordinator();
