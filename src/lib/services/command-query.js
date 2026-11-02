/** Debounce local searches and discard replies after the query or dialog changes.
 * @template T
 * @param {{ search: (query:string)=>Promise<T>, result:(value:T)=>void, error:(error:unknown)=>void, delay?:number }} options
 */
export function createCommandQuery(options) {
  let revision = 0;
  /** @type {ReturnType<typeof setTimeout> | undefined} */ let timer;
  function stop() { ++revision; clearTimeout(timer); }
  return {
    stop,
    /** @param {string} query */
    set(query) {
      stop(); const request = revision;
      if (!query.trim()) return;
      timer = setTimeout(() => { void options.search(query).then(value => {
        if (request === revision) options.result(value);
      }, error => { if (request === revision) options.error(error); }); }, options.delay ?? 180);
    },
  };
}
