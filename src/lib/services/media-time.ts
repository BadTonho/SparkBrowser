/** `m:ss`, or `h:mm:ss` from an hour up. Negative and fractional values are clamped and floored. */
export function formatClock(seconds: number): string {
  const total = Math.max(0, Math.floor(Number.isFinite(seconds) ? seconds : 0));
  const hours = Math.floor(total / 3600), minutes = Math.floor(total / 60) % 60, rest = total % 60;
  const pad = (value: number) => String(value).padStart(2, "0");
  return hours ? `${hours}:${pad(minutes)}:${pad(rest)}` : `${minutes}:${pad(rest)}`;
}

/** Position of a playing video, advanced by the time elapsed since it was sampled and capped at its length. */
export function elapsedSeconds(position: number, duration: number, playing: boolean, sampledAt: number, now: number): number {
  const advanced = playing ? Math.max(0, now - sampledAt) / 1000 : 0;
  return Math.min(duration, Math.max(0, position) + advanced);
}
