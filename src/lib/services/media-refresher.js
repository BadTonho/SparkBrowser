/** Shares in-flight native reports without letting one tab block another.
 * @template T
 * @param {(id:string)=>Promise<T>} fetch
 * @param {(snapshot:T)=>void} apply
 */
export function createMediaRefresher(fetch, apply) {
  /** @type {Map<string,Promise<T>>} */
  const pending = new Map();
  /** @param {string} id */
  return function refresh(id) {
    const current = pending.get(id);
    if (current) return current;
    const request = Promise.resolve().then(() => fetch(id)).then(snapshot => {
      apply(snapshot);
      return snapshot;
    }).finally(() => { pending.delete(id); });
    pending.set(id, request);
    return request;
  };
}
