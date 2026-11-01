/** Coalesce measurements to one frame and keep only the latest pending IPC.
 * @template T
 * @param {() => T | undefined} measure
 * @param {(value: T) => Promise<void>} send
 * @param {(error: unknown) => void} onError
 * @param {(callback: FrameRequestCallback) => number} frame
 * @param {(id: number) => void} cancel
 */
export function createViewportSync(measure, send, onError, frame = requestAnimationFrame, cancel = cancelAnimationFrame) {
  let frameId = 0;
  let disposed = false;
  let sending = false;
  /** @type {T | undefined} */
  let pending;
  let last = "";
  async function drain() {
    if (sending || disposed) return;
    sending = true;
    try {
      while (pending !== undefined && !disposed) {
        const value = pending;
        pending = undefined;
        const key = JSON.stringify(value);
        if (key === last) continue;
        try {
          await send(value);
          last = key;
        } catch (error) {
          onError(error);
        }
      }
    } finally {
      sending = false;
    }
  }
  return {
    schedule() {
      if (disposed || frameId) return;
      frameId = frame(() => {
        frameId = 0;
        pending = measure();
        void drain();
      });
    },
    dispose() {
      disposed = true;
      pending = undefined;
      if (frameId) cancel(frameId);
    },
  };
}
