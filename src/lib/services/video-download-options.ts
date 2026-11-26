import type { VideoSource } from "./bridge/video-download";

/**
 * What a video's card shows: only the options that can be saved. The reasons the others cannot
 * matter only when nothing is left, and each is said once.
 */
export function videoOptions(sources: VideoSource[], video: string): { options: VideoSource[]; reasons: string[] } {
  const own = sources.filter(source => source.video === video);
  const options = own.filter(source => !source.problem);
  const reasons = options.length ? [] : [...new Set(own.flatMap(source => source.problem ? [source.problem] : []))];
  return { options, reasons };
}
