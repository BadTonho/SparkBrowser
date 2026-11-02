/**
 * Coalesces each document's slider edits while preserving serialized native writes.
 * @template T
 * @param {(item: {id:string,document:string,value:number})=>Promise<T>} write
 * @param {(value:T)=>void} result
 * @param {(error:unknown,item:{id:string,document:string,value:number})=>void} error
 * @param {(item:{id:string,document:string,value:number},pending:boolean)=>void} settled
 */
export function createMediaVolumeWriter(write, result, error, settled) {
  /** @type {Map<string,{id:string,document:string,value:number}>} */
  const pending = new Map();
  /** @type {Promise<void>|null} */ let running = null;
  /** @type {ReturnType<typeof setTimeout>|undefined} */ let timer;
  function drain() {
    if (running) return running;
    running = (async () => {
      while (pending.size) {
        const entry = pending.entries().next().value;
        if (!entry) break;
        const [key, item] = entry;
        pending.delete(key);
        try { result(await write(item)); } catch (e) { error(e, item); }
        finally { settled(item, pending.has(key)); }
      }
    })().finally(() => { running = null; });
    return running;
  }
  return {
    /** @param {{id:string,document:string,value:number}} item */
    schedule(item) { pending.set(`${item.id}:${item.document}`, item); clearTimeout(timer); timer = setTimeout(drain, 60); },
    flush() { clearTimeout(timer); return drain(); },
  };
}
