/**
 * Waits for `work`, but never longer than `ms`. Used where a slow or failing step must not hold the
 * user up, such as the exit backup and sync while the browser is closing. It never rejects.
 * The work itself is not cancelled: it simply stops being waited for.
 * @template T
 * @param {Promise<T>} work
 * @param {number} ms
 * @returns {Promise<{ value: T } | { failed: true } | { timedOut: true }>}
 */
export async function settleWithin(work, ms) {
  /** @type {ReturnType<typeof setTimeout> | undefined} */
  let timer;
  const timeout = new Promise((resolve) => {
    timer = setTimeout(() => resolve({ timedOut: true }), ms);
  });
  try {
    return await Promise.race([work.then((value) => ({ value }), () => ({ failed: true })), timeout]);
  } finally {
    clearTimeout(timer);
  }
}
