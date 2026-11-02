import { describeError, UserError } from "./user-errors.js";
/** @typedef {{text:string,title:string,revision:number,status:string,error:string}} Draft */
/**
 * Each draft belongs to its original URL. A shared queue orders saves and deletions.
 * @param {{save:(url:string,title:string,text:string)=>Promise<void>,remove:(url:string)=>Promise<void>,changed:()=>void,delay?:number}} options
 */
export function createNoteWriter(options) {
  /** @type {Map<string,Draft>} */ const drafts = new Map();
  /** @type {Map<string,ReturnType<typeof setTimeout>>} */ const timers = new Map();
  let queue = Promise.resolve();
  /** @param {string} url @param {string} title @param {string} text */
  function seed(url, title, text) {
    if (!drafts.has(url)) drafts.set(url, { title, text, revision: 0, status: 'Saved', error: '' });
    return drafts.get(url);
  }
  /** @param {string} url */
  function flush(url) {
    clearTimeout(timers.get(url)); timers.delete(url);
    queue = queue.catch(() => {}).then(async () => {
      const draft = drafts.get(url); if (!draft || draft.status === 'Saved') return;
      const { revision, title, text } = draft;
      draft.status = 'Saving'; draft.error = ''; options.changed();
      try {
        await options.save(url, title, text);
        if (draft.revision === revision) draft.status = 'Saved';
      } catch (error) {
        draft.status = 'Error'; draft.error = describeError(error);
      }
      options.changed();
    });
    return queue;
  }
  return {
    drafts, seed, flush,
    /** Flush every URL before closing or updating; failed drafts remain recoverable. */
    async flushAll() {
      for (const url of drafts.keys()) await flush(url);
      await queue;
      if ([...drafts.values()].some(draft => draft.status !== 'Saved')) {
        throw new UserError('notes.flush_failed', 'Some page notes could not be saved. Retry saving them before closing.');
      }
    },
    /** @param {string} url @param {string} title @param {string} text */
    edit(url, title, text) {
      const draft = seed(url, title, text);
      if (!draft) return;
      Object.assign(draft, { title, text, revision: draft.revision + 1, status: 'Unsaved', error: '' });
      clearTimeout(timers.get(url)); timers.set(url, setTimeout(() => { void flush(url); }, options.delay ?? 500));
      options.changed();
    },
    /** @param {string} url */
    remove(url) {
      clearTimeout(timers.get(url)); timers.delete(url);
      const revision = drafts.get(url)?.revision;
      queue = queue.catch(() => {}).then(async () => {
        await options.remove(url);
        const draft = drafts.get(url);
        if (draft && draft.revision === revision) Object.assign(draft, { text: '', status: 'Saved', error: '' });
        options.changed();
      });
      return queue;
    },
  };
}
