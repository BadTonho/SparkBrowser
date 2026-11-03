export const AUTO_SKIP_STORAGE_KEY = 'veloce_auto_skip_ads';

/** @param {{getItem:(key:string)=>string|null}} storage */
export function readAutoSkip(storage) {
  try { return storage.getItem(AUTO_SKIP_STORAGE_KEY) === 'true'; } catch { return false; }
}

/** Confirm native and durable state before publishing a new preference.
 * @param {{storage:{getItem:(key:string)=>string|null,setItem:(key:string,value:string)=>unknown},apply:(enabled:boolean)=>Promise<boolean>,changed:(value:boolean,busy:boolean)=>void}} options
 */
export function createAutoSkipPreference({ storage, apply, changed }) {
  let confirmed = false, pending = 0, tail = Promise.resolve();
  /** @type {Promise<boolean>|undefined} */
  let initialization;
  const publish = () => changed(confirmed, pending > 0);
  /** @param {()=>boolean} request @param {boolean} persist */
  function submit(request, persist) {
    pending++; publish();
    const result = tail.then(async () => {
      const previous = confirmed;
      const enabled = request();
      await apply(enabled);
      try {
        if (persist && storage.setItem(AUTO_SKIP_STORAGE_KEY, String(enabled)) === false) throw Error('Could not save auto skip preference');
      } catch (error) {
        await apply(previous);
        throw error;
      }
      confirmed = enabled;
      return confirmed;
    }).finally(() => { pending--; publish(); });
    tail = result.then(() => {}, () => {});
    return result;
  }
  return {
    initialize() {
      initialization ??= submit(() => readAutoSkip(storage), false).catch(error => { initialization = undefined; throw error; });
      return initialization.then(() => tail);
    },
    /** @param {boolean} enabled */
    set(enabled) {
      if (typeof enabled !== 'boolean') return Promise.reject(new TypeError('Expected a boolean auto skip preference'));
      return submit(() => enabled, true);
    },
  };
}
