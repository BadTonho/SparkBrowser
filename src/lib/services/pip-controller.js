import { UserError } from './user-errors.js';

/** @typedef {import('../types/media').TabMediaState} TabMedia */
/** @typedef {{tabId:string,url:string,workspaceId:string,documentKey:string}} PipContext */
/** @typedef {{context:string,document:string,video:string}} PipVideo */

/** Uses intrinsic dimensions reported by the player; ties retain report order.
 * @param {TabMedia | undefined} media
 * @returns {PipVideo | null}
 */
export function selectPipVideo(media) {
  let area = -1;
  /** @type {PipVideo | null} */ let selected = null;
  for (const frame of media?.frames ?? []) {
    for (const video of frame.videos) {
      const nextArea = video.width * video.height;
      if (video.eligible && nextArea > area) {
        area = nextArea;
        selected = { context: frame.context, document: frame.document, video: video.id };
      }
    }
  }
  return selected;
}

/** @param {PipContext | null} before @param {PipContext | null} after */
function sameContext(before, after) {
  return before !== null && after !== null && before.tabId === after.tabId && before.url === after.url
    && before.workspaceId === after.workspaceId && before.documentKey === after.documentKey;
}

/** Keeps the click bound to its original focused document while awaiting native reports.
 * @param {{readContext:()=>PipContext|null,hasPip:()=>boolean,isBusy:()=>boolean,
 * refresh:(id:string)=>Promise<TabMedia|undefined>,closeVolume:()=>Promise<void>,
 * enter:(id:string,context:string,document:string,video:string)=>Promise<void>,exit:()=>Promise<void>,
 * onBusy:(busy:boolean)=>void,onError:(error:unknown)=>void}} options
 */
export function createPipController(options) {
  let busy = false;
  return async function toggle() {
    if (busy || options.isBusy()) return;
    busy = true;
    options.onBusy(true);
    const closing = options.hasPip();
    const context = options.readContext();
    try {
      await options.closeVolume();
      if (closing) { await options.exit(); return; }
      if (!context?.documentKey || !sameContext(context, options.readContext())) return;
      const media = await options.refresh(context.tabId);
      if (!sameContext(context, options.readContext())) return;
      const candidate = selectPipVideo(media);
      if (!candidate) throw new UserError('media.pip_unavailable', 'No eligible video in the focused document');
      await options.enter(context.tabId, candidate.context, candidate.document, candidate.video);
    } catch (error) { options.onError(error); }
    finally { busy = false; options.onBusy(false); }
  };
}
